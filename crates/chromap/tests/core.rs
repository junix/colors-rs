//! Integration tests for the public `chromap` API.

use chromap::{
    analogous_scale, average_color, best_black_or_white, composite_over, contrast_ratio,
    contrast_ratio_on, dominant_colors, ensure_contrast, format_color, gradient, harmony, mix,
    neighboring_lightness_scale, parse_color, Color, ColorFormat, CompositeSpace,
    ContrastDirection, Harmony, HexFormat, Hsl, HueInterpolation, MixSpace, Oklch,
};

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "actual={actual}, expected={expected}, tolerance={tolerance}"
    );
}

fn color_close(actual: Color, expected: Color, tolerance: f64) {
    close(actual.red(), expected.red(), tolerance);
    close(actual.green(), expected.green(), tolerance);
    close(actual.blue(), expected.blue(), tolerance);
    close(actual.alpha(), expected.alpha(), tolerance);
}

#[test]
fn parses_hex_functions_and_named_colors() {
    assert_eq!(parse_color("#0af").unwrap(), Color::from_rgb8(0, 170, 255));
    assert_eq!(
        parse_color("#0af8").unwrap(),
        Color::from_rgba8(0, 170, 255, 136)
    );
    assert_eq!(
        parse_color("rgb(100% 0% 50% / 25%)").unwrap(),
        Color::try_new(1.0, 0.0, 0.5, 0.25).unwrap()
    );
    assert_eq!(
        parse_color("rebeccapurple").unwrap(),
        Color::from_rgb8(102, 51, 153)
    );
    assert!(parse_color("not-a-color").is_err());
}

#[test]
fn hsl_known_values_round_trip() {
    let red = Color::from_rgb8(255, 0, 0);
    let hsl = red.to_hsl();
    close(hsl.h, 0.0, 1.0e-10);
    close(hsl.s, 1.0, 1.0e-10);
    close(hsl.l, 0.5, 1.0e-10);
    color_close(
        Color::from_hsl(Hsl::new(120.0, 1.0, 0.5).unwrap(), 1.0).unwrap(),
        Color::from_rgb8(0, 255, 0),
        1.0e-10,
    );
}

#[test]
fn oklab_round_trip_is_stable() {
    let source = Color::from_rgb8(36, 122, 211);
    let result = Color::try_from_oklab(source.to_oklab(), 1.0).unwrap();
    color_close(result, source, 2.0e-7);
}

#[test]
fn formatting_preserves_alpha() {
    let color = Color::from_rgba8(10, 20, 30, 128);
    assert_eq!(chromap::to_hex(color, HexFormat::Auto), "#0a141e80");
    assert_eq!(chromap::to_hex(color, HexFormat::Rgb), "#0a141e");
    assert!(format_color(color, ColorFormat::CssRgb).contains('/'));
}

#[test]
fn black_white_contrast_is_twenty_one() {
    close(
        contrast_ratio(Color::BLACK, Color::WHITE).unwrap(),
        21.0,
        1.0e-12,
    );
}

#[test]
fn black_or_white_is_measured() {
    let background = parse_color("#fff2a8").unwrap();
    let choice = best_black_or_white(background).unwrap();
    assert_eq!(choice.color, Color::BLACK);
    assert!(choice.ratio > 10.0);
}

#[test]
fn ensure_contrast_finds_darker_foreground() {
    let background = parse_color("#fff2a8").unwrap();
    let result = ensure_contrast(Color::WHITE, background, 4.5).unwrap();
    assert_eq!(result.direction, ContrastDirection::Darker);
    assert!(result.ratio >= 4.5);
}

#[test]
fn alpha_contrast_requires_canvas() {
    let foreground = Color::from_rgba8(0, 0, 0, 128);
    assert!(contrast_ratio(foreground, Color::WHITE).is_err());
    let ratio = contrast_ratio_on(foreground, Color::WHITE, Color::WHITE).unwrap();
    assert!(ratio > 3.9 && ratio < 4.1);
}

#[test]
fn gradient_is_inclusive() {
    let first = Color::from_rgb8(255, 0, 0);
    let second = Color::from_rgb8(0, 0, 255);
    let colors = gradient(first, second, 5, MixSpace::Oklab, HueInterpolation::Shorter).unwrap();
    assert_eq!(colors.len(), 5);
    assert_eq!(colors[0], first);
    assert_eq!(colors[4], second);
}

#[test]
fn oklch_short_path_crosses_zero() {
    let first = Color::from_oklch_mapped(Oklch::new(0.7, 0.05, 350.0).unwrap(), 1.0).unwrap();
    let second = Color::from_oklch_mapped(Oklch::new(0.7, 0.05, 10.0).unwrap(), 1.0).unwrap();
    let middle = mix(
        first,
        second,
        0.5,
        MixSpace::Oklch,
        HueInterpolation::Shorter,
    )
    .unwrap();
    let hue = middle.to_oklch().h;
    assert!(hue < 1.0 || hue > 359.0, "hue={hue}");
}

#[test]
fn palette_sizes_are_stable() {
    let base = parse_color("#4f7cff").unwrap();
    assert_eq!(harmony(base, Harmony::Complementary).unwrap().len(), 2);
    assert_eq!(analogous_scale(base, 9, 80.0).unwrap().len(), 9);
    assert_eq!(neighboring_lightness_scale(base, 7, 0.4).unwrap().len(), 7);
}

#[test]
fn source_over_srgb_has_expected_result() {
    let foreground = Color::from_rgba8(255, 0, 0, 128);
    let result = composite_over(foreground, Color::WHITE, CompositeSpace::Srgb);
    assert_eq!(
        result.to_rgb8(),
        chromap::Rgb8 {
            r: 255,
            g: 127,
            b: 127
        }
    );
}

#[test]
fn linear_average_is_not_naive_mid_gray() {
    let average = average_color(&[Color::BLACK, Color::WHITE]).unwrap();
    assert!((187..=188).contains(&average.to_rgb8().r));
}

#[test]
fn dominant_colors_sort_by_population() {
    let red = Color::from_rgb8(255, 0, 0);
    let blue = Color::from_rgb8(0, 0, 255);
    let swatches = dominant_colors(&[red, red, red, blue], 2, 16).unwrap();
    assert_eq!(swatches.len(), 2);
    assert_eq!(swatches[0].population, 3);
    assert_eq!(swatches[1].population, 1);
}
