use std::f64::consts::PI;

use crate::named::named_color;
use crate::{Color, ColorError, Hsl};

/// Parses a practical CSS-compatible color subset.
///
/// Supported forms: short/long hex with optional alpha, `0x` long hex,
/// `rgb()`/`rgba()`, `hsl()`/`hsla()`, CSS named colors, and `transparent`.
pub fn parse_color(input: &str) -> Result<Color, ColorError> {
    let input = input.trim();
    if input.is_empty() {
        return Err(ColorError::syntax(input, "the input is empty"));
    }
    let lower = input.to_ascii_lowercase();
    if let Some(color) = named_color(&lower) {
        return Ok(color);
    }
    if let Some(hex) = lower.strip_prefix('#') {
        return parse_hex(input, hex);
    }
    if let Some(hex) = lower.strip_prefix("0x") {
        return parse_hex(input, hex);
    }
    if let Some(body) = function_body(&lower, "rgb") {
        return parse_rgb(input, body, false);
    }
    if let Some(body) = function_body(&lower, "rgba") {
        return parse_rgb(input, body, true);
    }
    if let Some(body) = function_body(&lower, "hsl") {
        return parse_hsl(input, body, false);
    }
    if let Some(body) = function_body(&lower, "hsla") {
        return parse_hsl(input, body, true);
    }
    if lower.chars().all(|c| c.is_ascii_alphabetic()) {
        Err(ColorError::UnknownColorName(lower))
    } else {
        Err(ColorError::syntax(
            input,
            "expected hexadecimal, rgb(), hsl(), or a CSS named color",
        ))
    }
}

fn parse_hex(original: &str, digits: &str) -> Result<Color, ColorError> {
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ColorError::syntax(
            original,
            "hex colors may contain only 0-9 and a-f",
        ));
    }
    match digits.len() {
        3 => {
            let b = digits.as_bytes();
            Ok(Color::from_rgb8(
                nibble(b[0]) * 17,
                nibble(b[1]) * 17,
                nibble(b[2]) * 17,
            ))
        }
        4 => {
            let b = digits.as_bytes();
            Ok(Color::from_rgba8(
                nibble(b[0]) * 17,
                nibble(b[1]) * 17,
                nibble(b[2]) * 17,
                nibble(b[3]) * 17,
            ))
        }
        6 => Ok(Color::from_rgb8(
            pair(&digits[0..2])?,
            pair(&digits[2..4])?,
            pair(&digits[4..6])?,
        )),
        8 => Ok(Color::from_rgba8(
            pair(&digits[0..2])?,
            pair(&digits[2..4])?,
            pair(&digits[4..6])?,
            pair(&digits[6..8])?,
        )),
        _ => Err(ColorError::syntax(
            original,
            "hex colors require 3, 4, 6, or 8 digits",
        )),
    }
}

fn nibble(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        b'A'..=b'F' => byte - b'A' + 10,
        _ => 0,
    }
}

fn pair(value: &str) -> Result<u8, ColorError> {
    u8::from_str_radix(value, 16).map_err(|_| ColorError::syntax(value, "invalid hex byte"))
}

fn function_body<'a>(input: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}(");
    input
        .strip_prefix(&prefix)
        .and_then(|rest| rest.strip_suffix(')'))
}

fn parse_rgb(original: &str, body: &str, alpha_name: bool) -> Result<Color, ColorError> {
    let (mut channels, slash_alpha) = tokenize(original, body)?;
    let legacy_alpha = if slash_alpha.is_none() && channels.len() == 4 {
        if alpha_name {
            channels.pop()
        } else {
            return Err(ColorError::syntax(
                original,
                "four comma-separated values require rgba(), or use `/ alpha`",
            ));
        }
    } else {
        None
    };
    if channels.len() != 3 {
        return Err(ColorError::syntax(
            original,
            "rgb() requires three channels",
        ));
    }
    let alpha = slash_alpha.or(legacy_alpha).map_or(Ok(1.0), parse_alpha)?;
    Color::try_new(
        parse_rgb_channel(channels[0])?,
        parse_rgb_channel(channels[1])?,
        parse_rgb_channel(channels[2])?,
        alpha,
    )
}

fn parse_hsl(original: &str, body: &str, alpha_name: bool) -> Result<Color, ColorError> {
    let (mut channels, slash_alpha) = tokenize(original, body)?;
    let legacy_alpha = if slash_alpha.is_none() && channels.len() == 4 {
        if alpha_name {
            channels.pop()
        } else {
            return Err(ColorError::syntax(
                original,
                "four comma-separated values require hsla(), or use `/ alpha`",
            ));
        }
    } else {
        None
    };
    if channels.len() != 3 {
        return Err(ColorError::syntax(
            original,
            "hsl() requires hue, saturation, and lightness",
        ));
    }
    let alpha = slash_alpha.or(legacy_alpha).map_or(Ok(1.0), parse_alpha)?;
    Color::from_hsl(
        Hsl::new(
            parse_hue(channels[0])?,
            parse_percentage(channels[1], "saturation")?,
            parse_percentage(channels[2], "lightness")?,
        )?,
        alpha,
    )
}

fn tokenize<'a>(
    original: &str,
    body: &'a str,
) -> Result<(Vec<&'a str>, Option<&'a str>), ColorError> {
    let mut slash = body.split('/');
    let channels = slash.next().unwrap_or_default();
    let alpha = slash.next().map(str::trim);
    if slash.next().is_some() {
        return Err(ColorError::syntax(original, "at most one `/` is allowed"));
    }
    if alpha.is_some_and(str::is_empty) {
        return Err(ColorError::syntax(original, "alpha value is missing"));
    }
    let channels = channels
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|token| !token.is_empty())
        .collect();
    Ok((channels, alpha))
}

fn parse_rgb_channel(token: &str) -> Result<f64, ColorError> {
    if let Some(percent) = token.strip_suffix('%') {
        return bounded(percent, "RGB percentage", 0.0, 100.0).map(|v| v / 100.0);
    }
    bounded(token, "RGB channel", 0.0, 255.0).map(|v| v / 255.0)
}

fn parse_alpha(token: &str) -> Result<f64, ColorError> {
    if let Some(percent) = token.strip_suffix('%') {
        return bounded(percent, "alpha percentage", 0.0, 100.0).map(|v| v / 100.0);
    }
    bounded(token, "alpha", 0.0, 1.0)
}

fn parse_percentage(token: &str, component: &'static str) -> Result<f64, ColorError> {
    let Some(value) = token.strip_suffix('%') else {
        return Err(ColorError::syntax(
            token,
            format!("{component} must be a percentage"),
        ));
    };
    bounded(value, component, 0.0, 100.0).map(|v| v / 100.0)
}

fn parse_hue(token: &str) -> Result<f64, ColorError> {
    let (value, factor) = if let Some(v) = token.strip_suffix("turn") {
        (v, 360.0)
    } else if let Some(v) = token.strip_suffix("grad") {
        (v, 0.9)
    } else if let Some(v) = token.strip_suffix("rad") {
        (v, 180.0 / PI)
    } else if let Some(v) = token.strip_suffix("deg") {
        (v, 1.0)
    } else {
        (token, 1.0)
    };
    number(value, "hue").map(|v| v * factor)
}

fn bounded(token: &str, component: &'static str, min: f64, max: f64) -> Result<f64, ColorError> {
    let value = number(token, component)?;
    if (min..=max).contains(&value) {
        Ok(value)
    } else {
        Err(ColorError::OutOfRange {
            component,
            value,
            min,
            max,
        })
    }
}

fn number(token: &str, component: &'static str) -> Result<f64, ColorError> {
    let value = token
        .trim()
        .parse::<f64>()
        .map_err(|_| ColorError::syntax(token, format!("{component} is not a number")))?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(ColorError::syntax(
            token,
            format!("{component} must be finite"),
        ))
    }
}
