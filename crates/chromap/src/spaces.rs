use crate::color::{finite_clamp, validate_finite, validate_unit, EPSILON};
use crate::{Color, ColorError};

/// A linear-light sRGB color without alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LinearRgb {
    /// Linear red in `[0, 1]`.
    pub r: f64,
    /// Linear green in `[0, 1]`.
    pub g: f64,
    /// Linear blue in `[0, 1]`.
    pub b: f64,
}

impl LinearRgb {
    /// Creates an in-gamut linear sRGB value.
    pub fn new(r: f64, g: f64, b: f64) -> Result<Self, ColorError> {
        validate_unit("linear red", r)?;
        validate_unit("linear green", g)?;
        validate_unit("linear blue", b)?;
        Ok(Self { r, g, b })
    }
}

/// A hue-saturation-lightness color.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hsl {
    /// Hue in degrees.
    pub h: f64,
    /// Saturation in `[0, 1]`.
    pub s: f64,
    /// Lightness in `[0, 1]`.
    pub l: f64,
}

impl Hsl {
    /// Creates HSL and normalizes hue.
    pub fn new(h: f64, s: f64, l: f64) -> Result<Self, ColorError> {
        validate_finite("hue", h)?;
        validate_unit("saturation", s)?;
        validate_unit("lightness", l)?;
        Ok(Self {
            h: normalize_hue(h),
            s,
            l,
        })
    }
}

/// A hue-saturation-value color.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Hsv {
    /// Hue in degrees.
    pub h: f64,
    /// Saturation in `[0, 1]`.
    pub s: f64,
    /// Value in `[0, 1]`.
    pub v: f64,
}

impl Hsv {
    /// Creates HSV and normalizes hue.
    pub fn new(h: f64, s: f64, v: f64) -> Result<Self, ColorError> {
        validate_finite("hue", h)?;
        validate_unit("saturation", s)?;
        validate_unit("value", v)?;
        Ok(Self {
            h: normalize_hue(h),
            s,
            v,
        })
    }
}

/// A cyan-magenta-yellow-key value.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cmyk {
    /// Cyan in `[0, 1]`.
    pub c: f64,
    /// Magenta in `[0, 1]`.
    pub m: f64,
    /// Yellow in `[0, 1]`.
    pub y: f64,
    /// Key/black in `[0, 1]`.
    pub k: f64,
}

impl Cmyk {
    /// Creates a CMYK value.
    pub fn new(c: f64, m: f64, y: f64, k: f64) -> Result<Self, ColorError> {
        validate_unit("cyan", c)?;
        validate_unit("magenta", m)?;
        validate_unit("yellow", y)?;
        validate_unit("key", k)?;
        Ok(Self { c, m, y, k })
    }
}

/// An OKLab value using a D65 white point.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Oklab {
    /// Perceptual lightness.
    pub l: f64,
    /// Green-red opponent coordinate.
    pub a: f64,
    /// Blue-yellow opponent coordinate.
    pub b: f64,
}

impl Oklab {
    /// Creates an OKLab value suitable for display conversion.
    pub fn new(l: f64, a: f64, b: f64) -> Result<Self, ColorError> {
        validate_unit("OKLab lightness", l)?;
        validate_finite("OKLab a", a)?;
        validate_finite("OKLab b", b)?;
        Ok(Self { l, a, b })
    }
}

/// Cylindrical OKLab.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Oklch {
    /// Perceptual lightness in `[0, 1]`.
    pub l: f64,
    /// Chroma, greater than or equal to zero.
    pub c: f64,
    /// Hue in degrees.
    pub h: f64,
}

impl Oklch {
    /// Creates OKLCH and normalizes hue.
    pub fn new(l: f64, c: f64, h: f64) -> Result<Self, ColorError> {
        validate_unit("OKLCH lightness", l)?;
        if !c.is_finite() || c < 0.0 {
            return Err(ColorError::InvalidParameter {
                name: "OKLCH chroma",
                reason: "expected a finite value greater than or equal to zero",
            });
        }
        validate_finite("OKLCH hue", h)?;
        Ok(Self {
            l,
            c,
            h: normalize_hue(h),
        })
    }

    /// Converts to Cartesian OKLab.
    pub fn to_oklab(self) -> Oklab {
        let radians = self.h.to_radians();
        Oklab {
            l: self.l,
            a: self.c * radians.cos(),
            b: self.c * radians.sin(),
        }
    }

    /// Converts Cartesian OKLab to cylindrical form.
    pub fn from_oklab(value: Oklab) -> Self {
        let c = value.a.hypot(value.b);
        let h = if c <= EPSILON {
            0.0
        } else {
            normalize_hue(value.b.atan2(value.a).to_degrees())
        };
        Self { l: value.l, c, h }
    }
}

impl Color {
    /// Converts gamma-encoded sRGB to linear-light sRGB.
    pub fn to_linear_rgb(self) -> LinearRgb {
        LinearRgb {
            r: srgb_to_linear_component(self.red()),
            g: srgb_to_linear_component(self.green()),
            b: srgb_to_linear_component(self.blue()),
        }
    }

    /// Creates a color from in-gamut linear-light sRGB and alpha.
    pub fn from_linear_rgb(value: LinearRgb, alpha: f64) -> Result<Self, ColorError> {
        let value = LinearRgb::new(value.r, value.g, value.b)?;
        validate_unit("alpha", alpha)?;
        Ok(color_from_raw_linear(value, alpha))
    }

    /// Converts to HSL.
    pub fn to_hsl(self) -> Hsl {
        let max = self.red().max(self.green()).max(self.blue());
        let min = self.red().min(self.green()).min(self.blue());
        let delta = max - min;
        let l = (max + min) / 2.0;
        if delta <= EPSILON {
            return Hsl { h: 0.0, s: 0.0, l };
        }
        let s = delta / (1.0 - (2.0 * l - 1.0).abs());
        let sector = if (max - self.red()).abs() <= EPSILON {
            ((self.green() - self.blue()) / delta).rem_euclid(6.0)
        } else if (max - self.green()).abs() <= EPSILON {
            (self.blue() - self.red()) / delta + 2.0
        } else {
            (self.red() - self.green()) / delta + 4.0
        };
        Hsl {
            h: normalize_hue(sector * 60.0),
            s: s.clamp(0.0, 1.0),
            l,
        }
    }

    /// Creates a color from HSL and alpha.
    pub fn from_hsl(value: Hsl, alpha: f64) -> Result<Self, ColorError> {
        let value = Hsl::new(value.h, value.s, value.l)?;
        validate_unit("alpha", alpha)?;
        if value.s <= EPSILON {
            return Ok(Self::from_normalized_unchecked(
                value.l, value.l, value.l, alpha,
            ));
        }
        let q = if value.l < 0.5 {
            value.l * (1.0 + value.s)
        } else {
            value.l + value.s - value.l * value.s
        };
        let p = 2.0 * value.l - q;
        let hue = value.h / 360.0;
        Ok(Self::from_normalized_unchecked(
            hue_to_rgb(p, q, hue + 1.0 / 3.0),
            hue_to_rgb(p, q, hue),
            hue_to_rgb(p, q, hue - 1.0 / 3.0),
            alpha,
        ))
    }

    /// Converts to HSV.
    pub fn to_hsv(self) -> Hsv {
        let max = self.red().max(self.green()).max(self.blue());
        let min = self.red().min(self.green()).min(self.blue());
        let delta = max - min;
        if delta <= EPSILON {
            return Hsv {
                h: 0.0,
                s: 0.0,
                v: max,
            };
        }
        let sector = if (max - self.red()).abs() <= EPSILON {
            ((self.green() - self.blue()) / delta).rem_euclid(6.0)
        } else if (max - self.green()).abs() <= EPSILON {
            (self.blue() - self.red()) / delta + 2.0
        } else {
            (self.red() - self.green()) / delta + 4.0
        };
        Hsv {
            h: normalize_hue(sector * 60.0),
            s: if max <= EPSILON { 0.0 } else { delta / max },
            v: max,
        }
    }

    /// Creates a color from HSV and alpha.
    pub fn from_hsv(value: Hsv, alpha: f64) -> Result<Self, ColorError> {
        let value = Hsv::new(value.h, value.s, value.v)?;
        validate_unit("alpha", alpha)?;
        let chroma = value.v * value.s;
        let sector = value.h / 60.0;
        let x = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
        let (r1, g1, b1) = match sector.floor() as i32 {
            0 => (chroma, x, 0.0),
            1 => (x, chroma, 0.0),
            2 => (0.0, chroma, x),
            3 => (0.0, x, chroma),
            4 => (x, 0.0, chroma),
            _ => (chroma, 0.0, x),
        };
        let m = value.v - chroma;
        Ok(Self::from_normalized_unchecked(
            r1 + m,
            g1 + m,
            b1 + m,
            alpha,
        ))
    }

    /// Converts to a profile-free mathematical CMYK approximation.
    pub fn to_cmyk(self) -> Cmyk {
        let k = 1.0 - self.red().max(self.green()).max(self.blue());
        if k >= 1.0 - EPSILON {
            return Cmyk {
                c: 0.0,
                m: 0.0,
                y: 0.0,
                k: 1.0,
            };
        }
        let denominator = 1.0 - k;
        Cmyk {
            c: (1.0 - self.red() - k) / denominator,
            m: (1.0 - self.green() - k) / denominator,
            y: (1.0 - self.blue() - k) / denominator,
            k,
        }
    }

    /// Creates sRGB from the simple profile-free CMYK formula.
    pub fn from_cmyk(value: Cmyk, alpha: f64) -> Result<Self, ColorError> {
        let value = Cmyk::new(value.c, value.m, value.y, value.k)?;
        validate_unit("alpha", alpha)?;
        Ok(Self::from_normalized_unchecked(
            (1.0 - value.c) * (1.0 - value.k),
            (1.0 - value.m) * (1.0 - value.k),
            (1.0 - value.y) * (1.0 - value.k),
            alpha,
        ))
    }

    /// Converts to OKLab.
    pub fn to_oklab(self) -> Oklab {
        linear_to_oklab(self.to_linear_rgb())
    }
    /// Converts to OKLCH.
    pub fn to_oklch(self) -> Oklch {
        Oklch::from_oklab(self.to_oklab())
    }

    /// Converts OKLab only when it is already inside the sRGB gamut.
    pub fn try_from_oklab(value: Oklab, alpha: f64) -> Result<Self, ColorError> {
        let value = Oklab::new(value.l, value.a, value.b)?;
        validate_unit("alpha", alpha)?;
        let linear = oklab_to_linear(value);
        if !in_srgb_gamut(linear) {
            return Err(ColorError::OutOfGamut);
        }
        Ok(color_from_raw_linear(linear, alpha))
    }

    /// Converts OKLCH only when it is already inside the sRGB gamut.
    pub fn try_from_oklch(value: Oklch, alpha: f64) -> Result<Self, ColorError> {
        let value = Oklch::new(value.l, value.c, value.h)?;
        Self::try_from_oklab(value.to_oklab(), alpha)
    }

    /// Converts OKLab to sRGB, reducing chroma when needed.
    pub fn from_oklab_mapped(value: Oklab, alpha: f64) -> Result<Self, ColorError> {
        let value = Oklab::new(value.l, value.a, value.b)?;
        Self::from_oklch_mapped(Oklch::from_oklab(value), alpha)
    }

    /// Converts OKLCH with binary-search chroma reduction.
    ///
    /// Lightness and hue are retained as far as the sRGB gamut allows.
    pub fn from_oklch_mapped(value: Oklch, alpha: f64) -> Result<Self, ColorError> {
        let value = Oklch::new(value.l, value.c, value.h)?;
        validate_unit("alpha", alpha)?;
        let target = oklab_to_linear(value.to_oklab());
        if in_srgb_gamut(target) {
            return Ok(color_from_raw_linear(target, alpha));
        }
        let mut low = 0.0;
        let mut high = value.c;
        let mut best = oklab_to_linear(Oklch { c: 0.0, ..value }.to_oklab());
        for _ in 0..32 {
            let mid = (low + high) / 2.0;
            let candidate = oklab_to_linear(Oklch { c: mid, ..value }.to_oklab());
            if in_srgb_gamut(candidate) {
                low = mid;
                best = candidate;
            } else {
                high = mid;
            }
        }
        Ok(color_from_raw_linear(best, alpha))
    }
}

pub(crate) fn normalize_hue(hue: f64) -> f64 {
    hue.rem_euclid(360.0)
}

pub(crate) fn srgb_to_linear_component(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

pub(crate) fn linear_to_srgb_component(value: f64) -> f64 {
    if value <= 0.003_130_8 {
        12.92 * value
    } else {
        1.055 * value.powf(1.0 / 2.4) - 0.055
    }
}

pub(crate) fn linear_to_oklab(value: LinearRgb) -> Oklab {
    let l = 0.412_221_470_8 * value.r + 0.536_332_536_3 * value.g + 0.051_445_992_9 * value.b;
    let m = 0.211_903_498_2 * value.r + 0.680_699_545_1 * value.g + 0.107_396_956_6 * value.b;
    let s = 0.088_302_461_9 * value.r + 0.281_718_837_6 * value.g + 0.629_978_700_5 * value.b;
    let l_ = l.cbrt();
    let m_ = m.cbrt();
    let s_ = s.cbrt();
    Oklab {
        l: 0.210_454_255_3 * l_ + 0.793_617_785_0 * m_ - 0.004_072_046_8 * s_,
        a: 1.977_998_495_1 * l_ - 2.428_592_205_0 * m_ + 0.450_593_709_9 * s_,
        b: 0.025_904_037_1 * l_ + 0.782_771_766_2 * m_ - 0.808_675_766_0 * s_,
    }
}

pub(crate) fn oklab_to_linear(value: Oklab) -> LinearRgb {
    let l_ = value.l + 0.396_337_777_4 * value.a + 0.215_803_757_3 * value.b;
    let m_ = value.l - 0.105_561_345_8 * value.a - 0.063_854_172_8 * value.b;
    let s_ = value.l - 0.089_484_177_5 * value.a - 1.291_485_548_0 * value.b;
    let l = l_.powi(3);
    let m = m_.powi(3);
    let s = s_.powi(3);
    LinearRgb {
        r: 4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s,
        g: -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s,
        b: -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s,
    }
}

pub(crate) fn in_srgb_gamut(value: LinearRgb) -> bool {
    const T: f64 = 1.0e-7;
    value.r >= -T
        && value.r <= 1.0 + T
        && value.g >= -T
        && value.g <= 1.0 + T
        && value.b >= -T
        && value.b <= 1.0 + T
}

fn color_from_raw_linear(value: LinearRgb, alpha: f64) -> Color {
    Color::from_normalized_unchecked(
        finite_clamp(linear_to_srgb_component(value.r.clamp(0.0, 1.0))),
        finite_clamp(linear_to_srgb_component(value.g.clamp(0.0, 1.0))),
        finite_clamp(linear_to_srgb_component(value.b.clamp(0.0, 1.0))),
        finite_clamp(alpha),
    )
}

fn hue_to_rgb(p: f64, q: f64, hue: f64) -> f64 {
    let hue = hue.rem_euclid(1.0);
    if hue < 1.0 / 6.0 {
        p + (q - p) * 6.0 * hue
    } else if hue < 1.0 / 2.0 {
        q
    } else if hue < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - hue) * 6.0
    } else {
        p
    }
}
