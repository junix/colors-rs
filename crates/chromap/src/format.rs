use crate::Color;

/// Controls hexadecimal alpha emission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HexFormat {
    /// Emit six digits for opaque colors and eight otherwise.
    Auto,
    /// Always emit `#rrggbb`, omitting alpha.
    Rgb,
    /// Always emit `#rrggbbaa`.
    Rgba,
}

/// A human-readable output representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColorFormat {
    /// Hexadecimal.
    Hex,
    /// CSS `rgb()`.
    CssRgb,
    /// CSS `hsl()`.
    CssHsl,
    /// HSV diagnostic form.
    Hsv,
    /// CMYK diagnostic form.
    Cmyk,
    /// CSS-compatible `oklab()`.
    Oklab,
    /// CSS-compatible `oklch()`.
    Oklch,
}

/// Formats hexadecimal output.
pub fn to_hex(color: Color, format: HexFormat) -> String {
    let rgba = color.to_rgba8();
    match format {
        HexFormat::Rgb => format!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b),
        HexFormat::Auto if color.is_opaque() => {
            format!("#{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b)
        }
        HexFormat::Auto | HexFormat::Rgba => {
            format!("#{:02x}{:02x}{:02x}{:02x}", rgba.r, rgba.g, rgba.b, rgba.a)
        }
    }
}

/// Formats a color in the requested representation.
pub fn format_color(color: Color, format: ColorFormat) -> String {
    match format {
        ColorFormat::Hex => to_hex(color, HexFormat::Auto),
        ColorFormat::CssRgb => to_css_rgb(color),
        ColorFormat::CssHsl => to_css_hsl(color),
        ColorFormat::Hsv => to_hsv(color),
        ColorFormat::Cmyk => to_cmyk(color),
        ColorFormat::Oklab => to_css_oklab(color),
        ColorFormat::Oklch => to_css_oklch(color),
    }
}

/// Formats modern CSS `rgb()`.
pub fn to_css_rgb(color: Color) -> String {
    let rgba = color.to_rgba8();
    if color.is_opaque() {
        format!("rgb({} {} {})", rgba.r, rgba.g, rgba.b)
    } else {
        format!(
            "rgb({} {} {} / {})",
            rgba.r,
            rgba.g,
            rgba.b,
            decimal(color.alpha(), 4)
        )
    }
}

/// Formats modern CSS `hsl()`.
pub fn to_css_hsl(color: Color) -> String {
    let hsl = color.to_hsl();
    let body = format!(
        "{} {}% {}%",
        decimal(hsl.h, 3),
        decimal(hsl.s * 100.0, 3),
        decimal(hsl.l * 100.0, 3)
    );
    if color.is_opaque() {
        format!("hsl({body})")
    } else {
        format!("hsl({body} / {})", decimal(color.alpha(), 4))
    }
}

/// Formats an HSV diagnostic form.
pub fn to_hsv(color: Color) -> String {
    let hsv = color.to_hsv();
    format!(
        "hsv({} {}% {}% / {})",
        decimal(hsv.h, 3),
        decimal(hsv.s * 100.0, 3),
        decimal(hsv.v * 100.0, 3),
        decimal(color.alpha(), 4)
    )
}

/// Formats a profile-free CMYK diagnostic form.
pub fn to_cmyk(color: Color) -> String {
    let cmyk = color.to_cmyk();
    format!(
        "cmyk({}% {}% {}% {}% / {})",
        decimal(cmyk.c * 100.0, 3),
        decimal(cmyk.m * 100.0, 3),
        decimal(cmyk.y * 100.0, 3),
        decimal(cmyk.k * 100.0, 3),
        decimal(color.alpha(), 4)
    )
}

/// Formats CSS-compatible `oklab()`.
pub fn to_css_oklab(color: Color) -> String {
    let lab = color.to_oklab();
    let body = format!(
        "{}% {} {}",
        decimal(lab.l * 100.0, 4),
        decimal(lab.a, 6),
        decimal(lab.b, 6)
    );
    if color.is_opaque() {
        format!("oklab({body})")
    } else {
        format!("oklab({body} / {})", decimal(color.alpha(), 4))
    }
}

/// Formats CSS-compatible `oklch()`.
pub fn to_css_oklch(color: Color) -> String {
    let lch = color.to_oklch();
    let body = format!(
        "{}% {} {}",
        decimal(lch.l * 100.0, 4),
        decimal(lch.c, 6),
        decimal(lch.h, 4)
    );
    if color.is_opaque() {
        format!("oklch({body})")
    } else {
        format!("oklch({body} / {})", decimal(color.alpha(), 4))
    }
}

fn decimal(value: f64, precision: usize) -> String {
    let mut output = format!("{value:.prec$}", prec = precision);
    while output.contains('.') && output.ends_with('0') {
        output.pop();
    }
    if output.ends_with('.') {
        output.pop();
    }
    if output == "-0" {
        output = "0".to_owned();
    }
    output
}
