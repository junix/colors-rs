//! Color conversion, perceptual adjustment, palette, and accessibility tools.
//!
//! Color-wheel complements and readable foregrounds are separate concepts:
//! use [`harmony`] for hue relationships and [`best_foreground`] or
//! [`ensure_contrast`] for measured contrast.

#![forbid(unsafe_code)]

mod adjust;
mod color;
mod composite;
mod contrast;
mod difference;
mod error;
mod format;
mod mix;
mod named;
mod palette;
mod parse;
mod spaces;
mod stats;

pub use color::{Color, Rgb8, Rgba8};
pub use composite::{composite_over, flatten, CompositeSpace};
pub use contrast::{
    best_black_or_white, best_foreground, contrast_ratio, contrast_ratio_on, ensure_contrast,
    evaluate_contrast, evaluate_contrast_on, relative_luminance, ContrastAdjustment,
    ContrastChoice, ContrastDirection, ContrastRating, ContrastTarget,
};
pub use difference::{nearest_color, oklab_distance, srgb_distance};
pub use error::ColorError;
pub use format::{
    format_color, to_cmyk, to_css_hsl, to_css_oklab, to_css_oklch, to_css_rgb, to_hex, to_hsv,
    ColorFormat, HexFormat,
};
pub use mix::{gradient, mix, HueInterpolation, MixSpace};
pub use palette::{
    analogous_scale, golden_angle_palette, harmony, hue_wheel, lightness_scale,
    neighboring_lightness_scale, shades, tints, tones, Harmony,
};
pub use parse::parse_color;
pub use spaces::{Cmyk, Hsl, Hsv, LinearRgb, Oklab, Oklch};
pub use stats::{average_color, dominant_colors, Swatch};
