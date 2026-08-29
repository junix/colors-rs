use std::fmt;
use std::str::FromStr;

use crate::format::HexFormat;
use crate::{parse_color, ColorError};

pub(crate) const EPSILON: f64 = 1.0e-12;

/// A gamma-encoded sRGB color with alpha.
///
/// Components are finite normalized values in `[0, 1]`. RGB components are
/// gamma encoded, not linear-light values.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl Color {
    /// Opaque black.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// Opaque white.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    /// Fully transparent black.
    pub const TRANSPARENT: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };

    /// Creates a normalized sRGBA color after validating all components.
    pub fn try_new(r: f64, g: f64, b: f64, a: f64) -> Result<Self, ColorError> {
        validate_unit("red", r)?;
        validate_unit("green", g)?;
        validate_unit("blue", b)?;
        validate_unit("alpha", a)?;
        Ok(Self { r, g, b, a })
    }

    /// Creates an opaque normalized sRGB color after validation.
    pub fn try_rgb(r: f64, g: f64, b: f64) -> Result<Self, ColorError> {
        Self::try_new(r, g, b, 1.0)
    }

    /// Creates a color by clamping finite components to `[0, 1]`.
    ///
    /// Non-finite values become zero. Prefer [`Self::try_new`] at input
    /// boundaries where silent correction would hide an error.
    pub fn new_clamped(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self {
            r: finite_clamp(r),
            g: finite_clamp(g),
            b: finite_clamp(b),
            a: finite_clamp(a),
        }
    }

    /// Creates an opaque color from 8-bit sRGB channels.
    pub const fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Self::from_rgba8(r, g, b, 255)
    }

    /// Creates a color from 8-bit sRGBA channels.
    pub const fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: a as f64 / 255.0,
        }
    }

    /// Returns normalized red.
    pub const fn red(self) -> f64 {
        self.r
    }
    /// Returns normalized green.
    pub const fn green(self) -> f64 {
        self.g
    }
    /// Returns normalized blue.
    pub const fn blue(self) -> f64 {
        self.b
    }
    /// Returns normalized alpha.
    pub const fn alpha(self) -> f64 {
        self.a
    }
    /// Returns `(red, green, blue, alpha)`.
    pub const fn components(self) -> (f64, f64, f64, f64) {
        (self.r, self.g, self.b, self.a)
    }
    /// Returns whether alpha is effectively one.
    pub fn is_opaque(self) -> bool {
        (self.a - 1.0).abs() <= EPSILON
    }
    /// Returns whether alpha is effectively zero.
    pub fn is_transparent(self) -> bool {
        self.a.abs() <= EPSILON
    }

    /// Returns a copy with validated alpha.
    pub fn with_alpha(self, alpha: f64) -> Result<Self, ColorError> {
        validate_unit("alpha", alpha)?;
        Ok(Self { a: alpha, ..self })
    }

    /// Returns a copy with alpha clamped to `[0, 1]`.
    pub fn with_alpha_clamped(self, alpha: f64) -> Self {
        Self {
            a: finite_clamp(alpha),
            ..self
        }
    }

    /// Converts to rounded 8-bit RGB.
    pub fn to_rgb8(self) -> Rgb8 {
        Rgb8 {
            r: unit_to_u8(self.r),
            g: unit_to_u8(self.g),
            b: unit_to_u8(self.b),
        }
    }

    /// Converts to rounded 8-bit RGBA.
    pub fn to_rgba8(self) -> Rgba8 {
        Rgba8 {
            r: unit_to_u8(self.r),
            g: unit_to_u8(self.g),
            b: unit_to_u8(self.b),
            a: unit_to_u8(self.a),
        }
    }

    pub(crate) const fn from_normalized_unchecked(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let format = if self.is_opaque() {
            HexFormat::Rgb
        } else {
            HexFormat::Rgba
        };
        f.write_str(&crate::format::to_hex(*self, format))
    }
}

impl FromStr for Color {
    type Err = ColorError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse_color(input)
    }
}

/// An opaque 8-bit RGB value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rgb8 {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
}

impl From<Rgb8> for Color {
    fn from(value: Rgb8) -> Self {
        Self::from_rgb8(value.r, value.g, value.b)
    }
}

/// An 8-bit RGBA value.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rgba8 {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

impl From<Rgba8> for Color {
    fn from(value: Rgba8) -> Self {
        Self::from_rgba8(value.r, value.g, value.b, value.a)
    }
}

pub(crate) fn validate_unit(component: &'static str, value: f64) -> Result<(), ColorError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(ColorError::OutOfRange {
            component,
            value,
            min: 0.0,
            max: 1.0,
        })
    }
}

pub(crate) fn validate_finite(name: &'static str, value: f64) -> Result<(), ColorError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ColorError::InvalidParameter {
            name,
            reason: "expected a finite number",
        })
    }
}

pub(crate) fn finite_clamp(value: f64) -> f64 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

pub(crate) fn unit_to_u8(value: f64) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}
