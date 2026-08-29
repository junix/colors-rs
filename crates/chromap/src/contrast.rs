use crate::composite::{flatten, CompositeSpace};
use crate::{Color, ColorError, Oklab, Oklch};

/// Standard WCAG contrast targets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContrastTarget {
    /// AA large text/UI: `3:1`.
    AaLarge,
    /// AA normal text: `4.5:1`.
    AaNormal,
    /// AAA large text: `4.5:1`.
    AaaLarge,
    /// AAA normal text: `7:1`.
    AaaNormal,
}

impl ContrastTarget {
    /// Returns the numeric target.
    pub const fn ratio(self) -> f64 {
        match self {
            Self::AaLarge => 3.0,
            Self::AaNormal | Self::AaaLarge => 4.5,
            Self::AaaNormal => 7.0,
        }
    }
}

/// WCAG threshold evaluation.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContrastRating {
    /// Unrounded ratio.
    pub ratio: f64,
    /// Meets `3:1`.
    pub aa_large: bool,
    /// Meets `4.5:1`.
    pub aa_normal: bool,
    /// Meets `4.5:1`.
    pub aaa_large: bool,
    /// Meets `7:1`.
    pub aaa_normal: bool,
}

/// Selected foreground candidate.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContrastChoice {
    /// Selected color.
    pub color: Color,
    /// Index in the input list.
    pub index: usize,
    /// Measured ratio.
    pub ratio: f64,
    /// Whether the requested minimum is met.
    pub meets_minimum: bool,
}

/// Direction used to repair contrast.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ContrastDirection {
    /// No adjustment.
    Unchanged,
    /// Increased lightness.
    Lighter,
    /// Decreased lightness.
    Darker,
}

/// Result of automatic foreground repair.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContrastAdjustment {
    /// Original foreground.
    pub original: Color,
    /// Adjusted foreground.
    pub color: Color,
    /// Original ratio.
    pub original_ratio: f64,
    /// Final ratio.
    pub ratio: f64,
    /// Adjustment direction.
    pub direction: ContrastDirection,
}

/// Calculates WCAG relative luminance from gamma-encoded sRGB.
///
/// Alpha is ignored; resolve transparent colors against a canvas first.
pub fn relative_luminance(color: Color) -> f64 {
    let linear = color.to_linear_rgb();
    0.2126 * linear.r + 0.7152 * linear.g + 0.0722 * linear.b
}

/// Calculates contrast for two opaque colors.
pub fn contrast_ratio(foreground: Color, background: Color) -> Result<f64, ColorError> {
    if !foreground.is_opaque() || !background.is_opaque() {
        return Err(ColorError::AlphaRequiresCanvas);
    }
    Ok(raw_ratio(foreground, background))
}

/// Calculates contrast after resolving colors against an opaque canvas.
pub fn contrast_ratio_on(
    foreground: Color,
    background: Color,
    canvas: Color,
) -> Result<f64, ColorError> {
    let background = flatten(background, canvas, CompositeSpace::Srgb)?;
    let foreground = flatten(foreground, background, CompositeSpace::Srgb)?;
    Ok(raw_ratio(foreground, background))
}

/// Evaluates WCAG thresholds for opaque colors.
pub fn evaluate_contrast(
    foreground: Color,
    background: Color,
) -> Result<ContrastRating, ColorError> {
    Ok(rating(contrast_ratio(foreground, background)?))
}

/// Evaluates WCAG thresholds after alpha compositing.
pub fn evaluate_contrast_on(
    foreground: Color,
    background: Color,
    canvas: Color,
) -> Result<ContrastRating, ColorError> {
    Ok(rating(contrast_ratio_on(foreground, background, canvas)?))
}

/// Chooses the highest-contrast foreground candidate.
pub fn best_foreground(
    background: Color,
    candidates: &[Color],
    minimum: f64,
) -> Result<ContrastChoice, ColorError> {
    validate_target(minimum)?;
    if !background.is_opaque() {
        return Err(ColorError::AlphaRequiresCanvas);
    }
    if candidates.is_empty() {
        return Err(ColorError::EmptyInput("foreground candidates"));
    }
    let mut best: Option<ContrastChoice> = None;
    for (index, color) in candidates.iter().copied().enumerate() {
        if !color.is_opaque() {
            return Err(ColorError::AlphaRequiresCanvas);
        }
        let ratio = raw_ratio(color, background);
        let choice = ContrastChoice {
            color,
            index,
            ratio,
            meets_minimum: ratio >= minimum,
        };
        if match best {
            Some(current) => ratio > current.ratio,
            None => true,
        } {
            best = Some(choice);
        }
    }
    best.ok_or(ColorError::EmptyInput("foreground candidates"))
}

/// Chooses black or white by measured contrast.
pub fn best_black_or_white(background: Color) -> Result<ContrastChoice, ColorError> {
    best_foreground(background, &[Color::BLACK, Color::WHITE], 4.5)
}

/// Adjusts only OKLCH lightness until the foreground meets `minimum`.
pub fn ensure_contrast(
    foreground: Color,
    background: Color,
    minimum: f64,
) -> Result<ContrastAdjustment, ColorError> {
    validate_target(minimum)?;
    if !foreground.is_opaque() || !background.is_opaque() {
        return Err(ColorError::AlphaRequiresCanvas);
    }
    let original_ratio = raw_ratio(foreground, background);
    if original_ratio >= minimum {
        return Ok(ContrastAdjustment {
            original: foreground,
            color: foreground,
            original_ratio,
            ratio: original_ratio,
            direction: ContrastDirection::Unchanged,
        });
    }
    let lch = foreground.to_oklch();
    let lighter = search_lightness(background, lch, minimum, true)?;
    let darker = search_lightness(background, lch, minimum, false)?;
    let selected = match (lighter, darker) {
        (Some(left), Some(right)) => {
            if lab_distance(foreground.to_oklab(), left.0.to_oklab())
                <= lab_distance(foreground.to_oklab(), right.0.to_oklab())
            {
                (left, ContrastDirection::Lighter)
            } else {
                (right, ContrastDirection::Darker)
            }
        }
        (Some(candidate), None) => (candidate, ContrastDirection::Lighter),
        (None, Some(candidate)) => (candidate, ContrastDirection::Darker),
        (None, None) => {
            let maximum =
                raw_ratio(Color::BLACK, background).max(raw_ratio(Color::WHITE, background));
            return Err(ColorError::UnreachableContrast {
                target: minimum,
                maximum,
            });
        }
    };
    let ((color, ratio), direction) = selected;
    Ok(ContrastAdjustment {
        original: foreground,
        color,
        original_ratio,
        ratio,
        direction,
    })
}

fn rating(ratio: f64) -> ContrastRating {
    ContrastRating {
        ratio,
        aa_large: ratio >= 3.0,
        aa_normal: ratio >= 4.5,
        aaa_large: ratio >= 4.5,
        aaa_normal: ratio >= 7.0,
    }
}

fn raw_ratio(first: Color, second: Color) -> f64 {
    let first = relative_luminance(first);
    let second = relative_luminance(second);
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}

fn validate_target(target: f64) -> Result<(), ColorError> {
    if target.is_finite() && (1.0..=21.0).contains(&target) {
        Ok(())
    } else {
        Err(ColorError::InvalidContrastTarget(target))
    }
}

fn search_lightness(
    background: Color,
    base: Oklch,
    minimum: f64,
    lighter: bool,
) -> Result<Option<(Color, f64)>, ColorError> {
    let endpoint_l = if lighter { 1.0 } else { 0.0 };
    let endpoint = Color::from_oklch_mapped(
        Oklch {
            l: endpoint_l,
            ..base
        },
        1.0,
    )?;
    let endpoint_ratio = raw_ratio(endpoint, background);
    if endpoint_ratio < minimum {
        return Ok(None);
    }
    let (mut passing, mut failing) = if lighter {
        (1.0, base.l)
    } else {
        (0.0, base.l)
    };
    let mut best = (endpoint, endpoint_ratio);
    for _ in 0..48 {
        let middle = (passing + failing) / 2.0;
        let candidate = Color::from_oklch_mapped(Oklch { l: middle, ..base }, 1.0)?;
        let ratio = raw_ratio(candidate, background);
        if ratio >= minimum {
            passing = middle;
            best = (candidate, ratio);
        } else {
            failing = middle;
        }
    }
    Ok(Some(best))
}

fn lab_distance(first: Oklab, second: Oklab) -> f64 {
    ((first.l - second.l).powi(2) + (first.a - second.a).powi(2) + (first.b - second.b).powi(2))
        .sqrt()
}
