use crate::color::{validate_finite, validate_unit};
use crate::mix::{gradient, HueInterpolation, MixSpace};
use crate::{Color, ColorError, Oklch};

/// A fixed color-wheel relationship.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Harmony {
    /// Base and 180-degree complement.
    Complementary,
    /// Three nearby hues.
    Analogous,
    /// Base and 150/210-degree offsets.
    SplitComplementary,
    /// Three evenly spaced hues.
    Triadic,
    /// Four evenly spaced hues.
    Square,
    /// Rectangular four-color relationship.
    Tetradic,
}

/// Generates a fixed OKLCH harmony.
pub fn harmony(base: Color, kind: Harmony) -> Result<Vec<Color>, ColorError> {
    let offsets: &[f64] = match kind {
        Harmony::Complementary => &[0.0, 180.0],
        Harmony::Analogous => &[-30.0, 0.0, 30.0],
        Harmony::SplitComplementary => &[0.0, 150.0, 210.0],
        Harmony::Triadic => &[0.0, 120.0, 240.0],
        Harmony::Square => &[0.0, 90.0, 180.0, 270.0],
        Harmony::Tetradic => &[0.0, 60.0, 180.0, 240.0],
    };
    colors_at_offsets(base, offsets)
}

/// Generates evenly spaced hues.
pub fn hue_wheel(base: Color, count: usize) -> Result<Vec<Color>, ColorError> {
    validate_count("hue count", count)?;
    let step = 360.0 / count as f64;
    let offsets = (0..count).map(|i| i as f64 * step).collect::<Vec<_>>();
    colors_at_offsets(base, &offsets)
}

/// Generates nearby hues centered on the base.
pub fn analogous_scale(
    base: Color,
    count: usize,
    spread_degrees: f64,
) -> Result<Vec<Color>, ColorError> {
    validate_count("analogous count", count)?;
    validate_finite("hue spread", spread_degrees)?;
    if !(0.0..=360.0).contains(&spread_degrees) {
        return Err(ColorError::OutOfRange {
            component: "hue spread",
            value: spread_degrees,
            min: 0.0,
            max: 360.0,
        });
    }
    if count == 1 {
        return Ok(vec![base]);
    }
    let start = -spread_degrees / 2.0;
    let step = spread_degrees / (count - 1) as f64;
    let offsets = (0..count)
        .map(|i| start + i as f64 * step)
        .collect::<Vec<_>>();
    colors_at_offsets(base, &offsets)
}

/// Generates fixed-hue/chroma colors between lightness bounds.
pub fn lightness_scale(
    base: Color,
    count: usize,
    minimum: f64,
    maximum: f64,
) -> Result<Vec<Color>, ColorError> {
    validate_count("lightness count", count)?;
    validate_unit("minimum lightness", minimum)?;
    validate_unit("maximum lightness", maximum)?;
    if minimum > maximum {
        return Err(ColorError::InvalidParameter {
            name: "lightness bounds",
            reason: "minimum must be less than or equal to maximum",
        });
    }
    let lch = base.to_oklch();
    if count == 1 {
        return Ok(vec![Color::from_oklch_mapped(
            Oklch {
                l: (minimum + maximum) / 2.0,
                ..lch
            },
            base.alpha(),
        )?]);
    }
    (0..count)
        .map(|i| {
            let t = i as f64 / (count - 1) as f64;
            Color::from_oklch_mapped(
                Oklch {
                    l: minimum + (maximum - minimum) * t,
                    ..lch
                },
                base.alpha(),
            )
        })
        .collect()
}

/// Generates a lightness scale centered on the base.
pub fn neighboring_lightness_scale(
    base: Color,
    count: usize,
    span: f64,
) -> Result<Vec<Color>, ColorError> {
    validate_unit("lightness span", span)?;
    let center = base.to_oklch().l;
    lightness_scale(
        base,
        count,
        (center - span / 2.0).max(0.0),
        (center + span / 2.0).min(1.0),
    )
}

/// Generates a gradient from the base to white.
pub fn tints(base: Color, count: usize) -> Result<Vec<Color>, ColorError> {
    gradient(
        base,
        Color::WHITE.with_alpha(base.alpha())?,
        count,
        MixSpace::Oklab,
        HueInterpolation::Shorter,
    )
}

/// Generates a gradient from the base to black.
pub fn shades(base: Color, count: usize) -> Result<Vec<Color>, ColorError> {
    gradient(
        base,
        Color::BLACK.with_alpha(base.alpha())?,
        count,
        MixSpace::Oklab,
        HueInterpolation::Shorter,
    )
}

/// Generates a gradient from the base to equal-lightness gray.
pub fn tones(base: Color, count: usize) -> Result<Vec<Color>, ColorError> {
    let lch = base.to_oklch();
    let gray = Color::from_oklch_mapped(Oklch { c: 0.0, ..lch }, base.alpha())?;
    gradient(
        base,
        gray,
        count,
        MixSpace::Oklab,
        HueInterpolation::Shorter,
    )
}

/// Generates deterministic distinct hues using the golden angle.
pub fn golden_angle_palette(base: Color, count: usize) -> Result<Vec<Color>, ColorError> {
    validate_count("palette count", count)?;
    const GOLDEN_ANGLE: f64 = 137.507_764_050_037_85;
    let offsets = (0..count)
        .map(|i| i as f64 * GOLDEN_ANGLE)
        .collect::<Vec<_>>();
    colors_at_offsets(base, &offsets)
}

fn colors_at_offsets(base: Color, offsets: &[f64]) -> Result<Vec<Color>, ColorError> {
    let lch = base.to_oklch();
    offsets
        .iter()
        .copied()
        .map(|offset| {
            Color::from_oklch_mapped(
                Oklch {
                    h: lch.h + offset,
                    ..lch
                },
                base.alpha(),
            )
        })
        .collect()
}

fn validate_count(name: &'static str, count: usize) -> Result<(), ColorError> {
    if count == 0 {
        Err(ColorError::InvalidCount {
            name,
            value: count,
            minimum: 1,
        })
    } else {
        Ok(())
    }
}
