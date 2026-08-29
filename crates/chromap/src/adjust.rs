use crate::color::validate_finite;
use crate::{Color, ColorError, Hsl, Oklch};

impl Color {
    /// Adds a signed amount to OKLCH lightness and gamut maps the result.
    pub fn adjust_lightness(self, amount: f64) -> Result<Self, ColorError> {
        validate_finite("lightness adjustment", amount)?;
        let mut lch = self.to_oklch();
        lch.l = (lch.l + amount).clamp(0.0, 1.0);
        Self::from_oklch_mapped(lch, self.alpha())
    }

    /// Sets normalized OKLCH lightness.
    pub fn set_lightness(self, lightness: f64) -> Result<Self, ColorError> {
        let lch = self.to_oklch();
        Self::from_oklch_mapped(Oklch::new(lightness, lch.c, lch.h)?, self.alpha())
    }

    /// Scales OKLCH chroma by a non-negative factor.
    pub fn scale_chroma(self, factor: f64) -> Result<Self, ColorError> {
        if !factor.is_finite() || factor < 0.0 {
            return Err(ColorError::InvalidParameter {
                name: "chroma factor",
                reason: "expected a finite value greater than or equal to zero",
            });
        }
        let mut lch = self.to_oklch();
        lch.c *= factor;
        Self::from_oklch_mapped(lch, self.alpha())
    }

    /// Adjusts saturation relatively: `0.25` means +25%; `-1` removes chroma.
    pub fn adjust_saturation(self, amount: f64) -> Result<Self, ColorError> {
        if !amount.is_finite() || amount < -1.0 {
            return Err(ColorError::InvalidParameter {
                name: "saturation adjustment",
                reason: "expected a finite value greater than or equal to -1",
            });
        }
        self.scale_chroma(1.0 + amount)
    }

    /// Rotates OKLCH hue by degrees.
    pub fn rotate_hue(self, degrees: f64) -> Result<Self, ColorError> {
        validate_finite("hue rotation", degrees)?;
        let lch = self.to_oklch();
        Self::from_oklch_mapped(Oklch::new(lch.l, lch.c, lch.h + degrees)?, self.alpha())
    }

    /// Removes chroma while preserving OKLCH lightness.
    pub fn grayscale(self) -> Result<Self, ColorError> {
        let lch = self.to_oklch();
        Self::from_oklch_mapped(Oklch::new(lch.l, 0.0, lch.h)?, self.alpha())
    }

    /// Inverts gamma-encoded sRGB and preserves alpha.
    pub fn invert(self) -> Self {
        Self::from_normalized_unchecked(
            1.0 - self.red(),
            1.0 - self.green(),
            1.0 - self.blue(),
            self.alpha(),
        )
    }

    /// Adds a signed amount to alpha, clamping to `[0, 1]`.
    pub fn adjust_alpha(self, amount: f64) -> Result<Self, ColorError> {
        validate_finite("alpha adjustment", amount)?;
        Ok(self.with_alpha_clamped(self.alpha() + amount))
    }

    /// Adds a signed amount to traditional HSL lightness.
    pub fn adjust_hsl_lightness(self, amount: f64) -> Result<Self, ColorError> {
        validate_finite("HSL lightness adjustment", amount)?;
        let hsl = self.to_hsl();
        Self::from_hsl(
            Hsl::new(hsl.h, hsl.s, (hsl.l + amount).clamp(0.0, 1.0))?,
            self.alpha(),
        )
    }

    /// Adds a signed amount to traditional HSL saturation.
    pub fn adjust_hsl_saturation(self, amount: f64) -> Result<Self, ColorError> {
        validate_finite("HSL saturation adjustment", amount)?;
        let hsl = self.to_hsl();
        Self::from_hsl(
            Hsl::new(hsl.h, (hsl.s + amount).clamp(0.0, 1.0), hsl.l)?,
            self.alpha(),
        )
    }
}
