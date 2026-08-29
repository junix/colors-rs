use crate::spaces::linear_to_srgb_component;
use crate::{Color, ColorError, LinearRgb};

/// Color space used for source-over alpha compositing.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CompositeSpace {
    /// Gamma-encoded sRGB.
    #[default]
    Srgb,
    /// Linear-light sRGB.
    LinearSrgb,
}

/// Places `foreground` over `background`.
pub fn composite_over(foreground: Color, background: Color, space: CompositeSpace) -> Color {
    let source_alpha = foreground.alpha();
    let backdrop_alpha = background.alpha();
    let output_alpha = source_alpha + backdrop_alpha * (1.0 - source_alpha);
    if output_alpha <= f64::EPSILON {
        return Color::TRANSPARENT;
    }
    match space {
        CompositeSpace::Srgb => Color::new_clamped(
            channel(
                foreground.red(),
                background.red(),
                source_alpha,
                backdrop_alpha,
                output_alpha,
            ),
            channel(
                foreground.green(),
                background.green(),
                source_alpha,
                backdrop_alpha,
                output_alpha,
            ),
            channel(
                foreground.blue(),
                background.blue(),
                source_alpha,
                backdrop_alpha,
                output_alpha,
            ),
            output_alpha,
        ),
        CompositeSpace::LinearSrgb => {
            let source = foreground.to_linear_rgb();
            let backdrop = background.to_linear_rgb();
            let linear = LinearRgb {
                r: channel(
                    source.r,
                    backdrop.r,
                    source_alpha,
                    backdrop_alpha,
                    output_alpha,
                ),
                g: channel(
                    source.g,
                    backdrop.g,
                    source_alpha,
                    backdrop_alpha,
                    output_alpha,
                ),
                b: channel(
                    source.b,
                    backdrop.b,
                    source_alpha,
                    backdrop_alpha,
                    output_alpha,
                ),
            };
            Color::new_clamped(
                linear_to_srgb_component(linear.r),
                linear_to_srgb_component(linear.g),
                linear_to_srgb_component(linear.b),
                output_alpha,
            )
        }
    }
}

/// Resolves a color against an opaque canvas.
pub fn flatten(color: Color, canvas: Color, space: CompositeSpace) -> Result<Color, ColorError> {
    if !canvas.is_opaque() {
        return Err(ColorError::AlphaRequiresCanvas);
    }
    Ok(composite_over(color, canvas, space))
}

fn channel(source: f64, backdrop: f64, sa: f64, ba: f64, oa: f64) -> f64 {
    (source * sa + backdrop * ba * (1.0 - sa)) / oa
}
