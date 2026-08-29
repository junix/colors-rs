use crate::color::{validate_unit, EPSILON};
use crate::{Color, ColorError, LinearRgb, Oklab, Oklch};

/// Color space used for interpolation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MixSpace {
    /// Gamma-encoded sRGB.
    Srgb,
    /// Linear-light sRGB.
    LinearSrgb,
    /// Cartesian OKLab.
    #[default]
    Oklab,
    /// Cylindrical OKLCH.
    Oklch,
}

/// Hue path used for OKLCH interpolation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HueInterpolation {
    /// Shortest path.
    #[default]
    Shorter,
    /// Longest path.
    Longer,
    /// Increasing hue.
    Increasing,
    /// Decreasing hue.
    Decreasing,
}

/// Mixes two colors; `weight` is the contribution of `second`.
pub fn mix(
    first: Color,
    second: Color,
    weight: f64,
    space: MixSpace,
    hue_route: HueInterpolation,
) -> Result<Color, ColorError> {
    validate_unit("mix weight", weight)?;
    if weight <= EPSILON {
        return Ok(first);
    }
    if (1.0 - weight).abs() <= EPSILON {
        return Ok(second);
    }
    let alpha = lerp(first.alpha(), second.alpha(), weight);
    match space {
        MixSpace::Srgb => Ok(Color::from_normalized_unchecked(
            lerp(first.red(), second.red(), weight),
            lerp(first.green(), second.green(), weight),
            lerp(first.blue(), second.blue(), weight),
            alpha,
        )),
        MixSpace::LinearSrgb => {
            let left = first.to_linear_rgb();
            let right = second.to_linear_rgb();
            Color::from_linear_rgb(
                LinearRgb {
                    r: lerp(left.r, right.r, weight),
                    g: lerp(left.g, right.g, weight),
                    b: lerp(left.b, right.b, weight),
                },
                alpha,
            )
        }
        MixSpace::Oklab => {
            let left = first.to_oklab();
            let right = second.to_oklab();
            Color::from_oklab_mapped(
                Oklab {
                    l: lerp(left.l, right.l, weight),
                    a: lerp(left.a, right.a, weight),
                    b: lerp(left.b, right.b, weight),
                },
                alpha,
            )
        }
        MixSpace::Oklch => {
            let mut left = first.to_oklch();
            let mut right = second.to_oklch();
            if left.c <= EPSILON {
                left.h = right.h;
            }
            if right.c <= EPSILON {
                right.h = left.h;
            }
            let delta = hue_delta(left.h, right.h, hue_route);
            Color::from_oklch_mapped(
                Oklch {
                    l: lerp(left.l, right.l, weight),
                    c: lerp(left.c, right.c, weight),
                    h: left.h + delta * weight,
                },
                alpha,
            )
        }
    }
}

/// Generates an inclusive gradient. A count of one returns `first`.
pub fn gradient(
    first: Color,
    second: Color,
    count: usize,
    space: MixSpace,
    hue_route: HueInterpolation,
) -> Result<Vec<Color>, ColorError> {
    if count == 0 {
        return Err(ColorError::InvalidCount {
            name: "gradient count",
            value: count,
            minimum: 1,
        });
    }
    if count == 1 {
        return Ok(vec![first]);
    }
    (0..count)
        .map(|index| {
            mix(
                first,
                second,
                index as f64 / (count - 1) as f64,
                space,
                hue_route,
            )
        })
        .collect()
}

fn lerp(first: f64, second: f64, weight: f64) -> f64 {
    first + (second - first) * weight
}

fn hue_delta(first: f64, second: f64, route: HueInterpolation) -> f64 {
    let increasing = (second - first).rem_euclid(360.0);
    let decreasing = -((first - second).rem_euclid(360.0));
    match route {
        HueInterpolation::Increasing => increasing,
        HueInterpolation::Decreasing => decreasing,
        HueInterpolation::Shorter => {
            if increasing <= 180.0 {
                increasing
            } else {
                decreasing
            }
        }
        HueInterpolation::Longer => {
            if increasing > 180.0 {
                increasing
            } else if decreasing.abs() > EPSILON {
                decreasing
            } else {
                360.0
            }
        }
    }
}
