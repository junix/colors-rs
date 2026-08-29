//! CLI for the `chromap` color toolkit.

use std::error::Error;
use std::io::{Error as IoError, ErrorKind};
use std::process::ExitCode;

use chromap::{
    analogous_scale, average_color, best_black_or_white, best_foreground, composite_over,
    contrast_ratio, dominant_colors, ensure_contrast, evaluate_contrast, evaluate_contrast_on,
    format_color, golden_angle_palette, gradient, harmony, hue_wheel, lightness_scale, mix,
    neighboring_lightness_scale, oklab_distance, relative_luminance, shades, srgb_distance, tints,
    tones, Color, ColorFormat, CompositeSpace, ContrastRating, Harmony, HexFormat,
    HueInterpolation, MixSpace,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{json, Value};

#[derive(Debug, Parser)]
#[command(
    name = "chromap",
    version,
    about = "Color conversion, perceptual palettes, and WCAG contrast tools",
    propagate_version = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Color representation used by color-emitting commands.
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Hex)]
    format: OutputFormat,

    /// Emit structured JSON.
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Show conversions and measurements.
    Inspect { color: Color },
    /// Convert one color to the global --format.
    Convert { color: Color },
    /// Adjust a color.
    Adjust(AdjustArgs),
    /// Mix two colors.
    Mix(MixArgs),
    /// Generate an inclusive gradient.
    Gradient(GradientArgs),
    /// Generate a harmony or scale.
    Palette(PaletteArgs),
    /// Measure, select, or repair contrast.
    Contrast {
        #[command(subcommand)]
        command: ContrastCommand,
    },
    /// Measure OKLab and sRGB distances.
    Distance { first: Color, second: Color },
    /// Alpha-composite foreground over background.
    Composite {
        foreground: Color,
        background: Color,
        #[arg(long, value_enum, default_value_t = CompositeSpaceArg::Srgb)]
        space: CompositeSpaceArg,
    },
    /// Compute an alpha-aware average.
    Average {
        #[arg(required = true, num_args = 1..)]
        colors: Vec<Color>,
    },
    /// Cluster supplied colors into representative swatches.
    Dominant {
        #[arg(short, long, default_value_t = 5)]
        count: usize,
        #[arg(long, default_value_t = 32)]
        iterations: usize,
        #[arg(required = true, num_args = 1..)]
        colors: Vec<Color>,
    },
}

#[derive(Debug, Args)]
struct AdjustArgs {
    color: Color,
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    lightness: f64,
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    saturation: f64,
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    hue: f64,
    #[arg(long, default_value_t = 0.0, allow_hyphen_values = true)]
    alpha: f64,
    #[arg(long, value_enum, default_value_t = AdjustmentSpace::Oklch)]
    space: AdjustmentSpace,
    #[arg(long)]
    grayscale: bool,
    #[arg(long)]
    invert: bool,
}

#[derive(Debug, Args)]
struct MixArgs {
    first: Color,
    second: Color,
    #[arg(short, long, default_value_t = 0.5)]
    weight: f64,
    #[arg(long, value_enum, default_value_t = MixSpaceArg::Oklab)]
    space: MixSpaceArg,
    #[arg(long, value_enum, default_value_t = HueRouteArg::Shorter)]
    hue_route: HueRouteArg,
}

#[derive(Debug, Args)]
struct GradientArgs {
    first: Color,
    second: Color,
    #[arg(short, long, default_value_t = 7)]
    steps: usize,
    #[arg(long, value_enum, default_value_t = MixSpaceArg::Oklab)]
    space: MixSpaceArg,
    #[arg(long, value_enum, default_value_t = HueRouteArg::Shorter)]
    hue_route: HueRouteArg,
    #[arg(long)]
    css_prefix: Option<String>,
}

#[derive(Debug, Args)]
struct PaletteArgs {
    color: Color,
    #[arg(short, long, value_enum, default_value_t = PaletteKind::Neighbors)]
    kind: PaletteKind,
    #[arg(short, long, default_value_t = 7)]
    count: usize,
    #[arg(long, default_value_t = 0.4)]
    lightness_span: f64,
    #[arg(long, default_value_t = 60.0)]
    hue_span: f64,
    #[arg(long, default_value_t = 0.1)]
    min_lightness: f64,
    #[arg(long, default_value_t = 0.9)]
    max_lightness: f64,
    #[arg(long)]
    css_prefix: Option<String>,
}

#[derive(Debug, Subcommand)]
enum ContrastCommand {
    /// Measure ratio and WCAG thresholds.
    Ratio {
        foreground: Color,
        background: Color,
        #[arg(long)]
        canvas: Option<Color>,
    },
    /// Pick the highest-contrast candidate.
    Pick {
        background: Color,
        #[arg(required = true, num_args = 1..)]
        candidates: Vec<Color>,
        #[arg(short, long, default_value_t = 4.5)]
        minimum: f64,
    },
    /// Choose black or white.
    BlackWhite { background: Color },
    /// Change only foreground lightness until a target is met.
    Ensure {
        foreground: Color,
        background: Color,
        #[arg(long, value_enum, default_value_t = TargetArg::Aa)]
        target: TargetArg,
        #[arg(long)]
        minimum: Option<f64>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Hex,
    Rgb,
    Hsl,
    Hsv,
    Cmyk,
    Oklab,
    Oklch,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
enum AdjustmentSpace {
    Oklch,
    Hsl,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
enum MixSpaceArg {
    Srgb,
    LinearSrgb,
    Oklab,
    Oklch,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
enum HueRouteArg {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
enum CompositeSpaceArg {
    Srgb,
    LinearSrgb,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
enum PaletteKind {
    Neighbors,
    Lightness,
    AnalogousScale,
    HueWheel,
    Golden,
    Tints,
    Shades,
    Tones,
    Complementary,
    Analogous,
    SplitComplementary,
    Triadic,
    Square,
    Tetradic,
}
#[derive(Clone, Copy, Debug, ValueEnum)]
enum TargetArg {
    AaLarge,
    Aa,
    AaaLarge,
    Aaa,
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    let Cli {
        command,
        format,
        json,
    } = cli;
    match command {
        Command::Inspect { color } => inspect(color, json),
        Command::Convert { color } => print_one(color, format, json),
        Command::Adjust(args) => print_one(adjust(args)?, format, json),
        Command::Mix(args) => print_one(
            mix(
                args.first,
                args.second,
                args.weight,
                args.space.into(),
                args.hue_route.into(),
            )?,
            format,
            json,
        ),
        Command::Gradient(args) => {
            let colors = gradient(
                args.first,
                args.second,
                args.steps,
                args.space.into(),
                args.hue_route.into(),
            )?;
            print_many(&colors, format, json, args.css_prefix.as_deref())
        }
        Command::Palette(args) => {
            let colors = generate_palette(&args)?;
            print_many(&colors, format, json, args.css_prefix.as_deref())
        }
        Command::Contrast { command } => run_contrast(command, format, json),
        Command::Distance { first, second } => {
            let lab = oklab_distance(first, second);
            let rgb = srgb_distance(first, second);
            if json {
                print_json(&json!({ "oklab": lab, "srgb": rgb }))
            } else {
                println!("OKLab distance: {lab:.8}");
                println!("sRGB distance:  {rgb:.8}");
                Ok(())
            }
        }
        Command::Composite {
            foreground,
            background,
            space,
        } => print_one(
            composite_over(foreground, background, space.into()),
            format,
            json,
        ),
        Command::Average { colors } => print_one(average_color(&colors)?, format, json),
        Command::Dominant {
            count,
            iterations,
            colors,
        } => {
            let swatches = dominant_colors(&colors, count, iterations)?;
            if json {
                print_json(&Value::Array(
                    swatches
                        .iter()
                        .map(|s| {
                            json!({
                                "color": color_value(s.color),
                                "population": s.population,
                                "weight": s.weight,
                            })
                        })
                        .collect(),
                ))
            } else {
                for swatch in swatches {
                    println!(
                        "{}\tpopulation={}\tweight={:.6}",
                        render_color(swatch.color, format),
                        swatch.population,
                        swatch.weight
                    );
                }
                Ok(())
            }
        }
    }
}

fn adjust(args: AdjustArgs) -> Result<Color, Box<dyn Error>> {
    let mut color = args.color;
    if args.invert {
        color = color.invert();
    }
    if args.grayscale {
        color = color.grayscale()?;
    }
    color = match args.space {
        AdjustmentSpace::Oklch => color
            .adjust_lightness(args.lightness)?
            .adjust_saturation(args.saturation)?,
        AdjustmentSpace::Hsl => color
            .adjust_hsl_lightness(args.lightness)?
            .adjust_hsl_saturation(args.saturation)?,
    };
    color = color.rotate_hue(args.hue)?;
    Ok(color.adjust_alpha(args.alpha)?)
}

fn generate_palette(args: &PaletteArgs) -> Result<Vec<Color>, Box<dyn Error>> {
    Ok(match args.kind {
        PaletteKind::Neighbors => {
            neighboring_lightness_scale(args.color, args.count, args.lightness_span)?
        }
        PaletteKind::Lightness => lightness_scale(
            args.color,
            args.count,
            args.min_lightness,
            args.max_lightness,
        )?,
        PaletteKind::AnalogousScale => analogous_scale(args.color, args.count, args.hue_span)?,
        PaletteKind::HueWheel => hue_wheel(args.color, args.count)?,
        PaletteKind::Golden => golden_angle_palette(args.color, args.count)?,
        PaletteKind::Tints => tints(args.color, args.count)?,
        PaletteKind::Shades => shades(args.color, args.count)?,
        PaletteKind::Tones => tones(args.color, args.count)?,
        PaletteKind::Complementary => harmony(args.color, Harmony::Complementary)?,
        PaletteKind::Analogous => harmony(args.color, Harmony::Analogous)?,
        PaletteKind::SplitComplementary => harmony(args.color, Harmony::SplitComplementary)?,
        PaletteKind::Triadic => harmony(args.color, Harmony::Triadic)?,
        PaletteKind::Square => harmony(args.color, Harmony::Square)?,
        PaletteKind::Tetradic => harmony(args.color, Harmony::Tetradic)?,
    })
}

fn run_contrast(
    command: ContrastCommand,
    format: OutputFormat,
    json_output: bool,
) -> Result<(), Box<dyn Error>> {
    match command {
        ContrastCommand::Ratio {
            foreground,
            background,
            canvas,
        } => {
            let rating = match canvas {
                Some(canvas) => evaluate_contrast_on(foreground, background, canvas)?,
                None => evaluate_contrast(foreground, background)?,
            };
            print_rating(rating, json_output)
        }
        ContrastCommand::Pick {
            background,
            candidates,
            minimum,
        } => {
            let choice = best_foreground(background, &candidates, minimum)?;
            if json_output {
                print_json(&json!({
                    "color": color_value(choice.color),
                    "index": choice.index,
                    "ratio": choice.ratio,
                    "minimum": minimum,
                    "meets_minimum": choice.meets_minimum,
                }))
            } else {
                println!("{}", render_color(choice.color, format));
                println!("index: {}", choice.index);
                println!("ratio: {:.6}:1", choice.ratio);
                println!("meets minimum: {}", choice.meets_minimum);
                Ok(())
            }
        }
        ContrastCommand::BlackWhite { background } => {
            let choice = best_black_or_white(background)?;
            if json_output {
                print_json(&json!({
                    "color": color_value(choice.color),
                    "ratio": choice.ratio,
                }))
            } else {
                println!("{}", render_color(choice.color, format));
                println!("ratio: {:.6}:1", choice.ratio);
                Ok(())
            }
        }
        ContrastCommand::Ensure {
            foreground,
            background,
            target,
            minimum,
        } => {
            let minimum = minimum.unwrap_or(target.ratio());
            let result = ensure_contrast(foreground, background, minimum)?;
            if json_output {
                print_json(&json!({
                    "original": color_value(result.original),
                    "color": color_value(result.color),
                    "original_ratio": result.original_ratio,
                    "ratio": result.ratio,
                    "minimum": minimum,
                    "direction": format!("{:?}", result.direction).to_ascii_lowercase(),
                }))
            } else {
                println!("{}", render_color(result.color, format));
                println!("original ratio: {:.6}:1", result.original_ratio);
                println!("final ratio: {:.6}:1", result.ratio);
                println!("direction: {:?}", result.direction);
                Ok(())
            }
        }
    }
}

fn inspect(color: Color, json_output: bool) -> Result<(), Box<dyn Error>> {
    if json_output {
        print_json(&json!({
            "color": color_value(color),
            "relative_luminance": relative_luminance(color),
            "contrast_on_black": if color.is_opaque() {
                Some(contrast_ratio(color, Color::BLACK)?)
            } else { None },
            "contrast_on_white": if color.is_opaque() {
                Some(contrast_ratio(color, Color::WHITE)?)
            } else { None },
        }))
    } else {
        println!("hex:    {}", format_color(color, ColorFormat::Hex));
        println!("rgb:    {}", format_color(color, ColorFormat::CssRgb));
        println!("hsl:    {}", format_color(color, ColorFormat::CssHsl));
        println!("hsv:    {}", format_color(color, ColorFormat::Hsv));
        println!("cmyk:   {}", format_color(color, ColorFormat::Cmyk));
        println!("oklab:  {}", format_color(color, ColorFormat::Oklab));
        println!("oklch:  {}", format_color(color, ColorFormat::Oklch));
        println!("luminance: {:.8}", relative_luminance(color));
        if color.is_opaque() {
            println!(
                "contrast on black: {:.6}:1",
                contrast_ratio(color, Color::BLACK)?
            );
            println!(
                "contrast on white: {:.6}:1",
                contrast_ratio(color, Color::WHITE)?
            );
        }
        Ok(())
    }
}

fn print_one(color: Color, format: OutputFormat, json_output: bool) -> Result<(), Box<dyn Error>> {
    if json_output {
        print_json(&color_value(color))
    } else {
        println!("{}", render_color(color, format));
        Ok(())
    }
}

fn print_many(
    colors: &[Color],
    format: OutputFormat,
    json_output: bool,
    css_prefix: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    if json_output {
        let mut object = serde_json::Map::new();
        object.insert(
            "colors".to_owned(),
            Value::Array(colors.iter().copied().map(color_value).collect()),
        );
        if let Some(prefix) = css_prefix {
            object.insert(
                "css".to_owned(),
                Value::String(css_variables(prefix, colors)?),
            );
        }
        print_json(&Value::Object(object))
    } else {
        for color in colors.iter().copied() {
            println!("{}", render_color(color, format));
        }
        if let Some(prefix) = css_prefix {
            println!();
            println!("{}", css_variables(prefix, colors)?);
        }
        Ok(())
    }
}

fn print_rating(rating: ContrastRating, json_output: bool) -> Result<(), Box<dyn Error>> {
    if json_output {
        print_json(&json!({
            "ratio": rating.ratio,
            "aa_large": rating.aa_large,
            "aa_normal": rating.aa_normal,
            "aaa_large": rating.aaa_large,
            "aaa_normal": rating.aaa_normal,
        }))
    } else {
        println!("ratio: {:.6}:1", rating.ratio);
        println!("AA large/UI (3:1): {}", rating.aa_large);
        println!("AA normal (4.5:1): {}", rating.aa_normal);
        println!("AAA large (4.5:1): {}", rating.aaa_large);
        println!("AAA normal (7:1): {}", rating.aaa_normal);
        Ok(())
    }
}

fn print_json(value: &Value) -> Result<(), Box<dyn Error>> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn render_color(color: Color, format: OutputFormat) -> String {
    format_color(
        color,
        match format {
            OutputFormat::Hex => ColorFormat::Hex,
            OutputFormat::Rgb => ColorFormat::CssRgb,
            OutputFormat::Hsl => ColorFormat::CssHsl,
            OutputFormat::Hsv => ColorFormat::Hsv,
            OutputFormat::Cmyk => ColorFormat::Cmyk,
            OutputFormat::Oklab => ColorFormat::Oklab,
            OutputFormat::Oklch => ColorFormat::Oklch,
        },
    )
}

fn color_value(color: Color) -> Value {
    let rgba = color.to_rgba8();
    json!({
        "hex": chromap::to_hex(color, HexFormat::Auto),
        "rgb": format_color(color, ColorFormat::CssRgb),
        "hsl": format_color(color, ColorFormat::CssHsl),
        "hsv": format_color(color, ColorFormat::Hsv),
        "cmyk": format_color(color, ColorFormat::Cmyk),
        "oklab": format_color(color, ColorFormat::Oklab),
        "oklch": format_color(color, ColorFormat::Oklch),
        "rgba8": { "r": rgba.r, "g": rgba.g, "b": rgba.b, "a": rgba.a },
        "alpha": color.alpha(),
    })
}

fn css_variables(prefix: &str, colors: &[Color]) -> Result<String, Box<dyn Error>> {
    let prefix = prefix.trim();
    if prefix.is_empty()
        || !prefix
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(IoError::new(
            ErrorKind::InvalidInput,
            "CSS prefix must contain only ASCII letters, digits, '-' or '_'",
        )
        .into());
    }
    let mut output = String::from(":root {\n");
    for (index, color) in colors.iter().copied().enumerate() {
        output.push_str(&format!(
            "  --{prefix}-{}: {};\n",
            index + 1,
            chromap::to_hex(color, HexFormat::Auto)
        ));
    }
    output.push('}');
    Ok(output)
}

impl From<MixSpaceArg> for MixSpace {
    fn from(value: MixSpaceArg) -> Self {
        match value {
            MixSpaceArg::Srgb => Self::Srgb,
            MixSpaceArg::LinearSrgb => Self::LinearSrgb,
            MixSpaceArg::Oklab => Self::Oklab,
            MixSpaceArg::Oklch => Self::Oklch,
        }
    }
}

impl From<HueRouteArg> for HueInterpolation {
    fn from(value: HueRouteArg) -> Self {
        match value {
            HueRouteArg::Shorter => Self::Shorter,
            HueRouteArg::Longer => Self::Longer,
            HueRouteArg::Increasing => Self::Increasing,
            HueRouteArg::Decreasing => Self::Decreasing,
        }
    }
}

impl From<CompositeSpaceArg> for CompositeSpace {
    fn from(value: CompositeSpaceArg) -> Self {
        match value {
            CompositeSpaceArg::Srgb => Self::Srgb,
            CompositeSpaceArg::LinearSrgb => Self::LinearSrgb,
        }
    }
}

impl TargetArg {
    const fn ratio(self) -> f64 {
        match self {
            Self::AaLarge => 3.0,
            Self::Aa | Self::AaaLarge => 4.5,
            Self::Aaa => 7.0,
        }
    }
}
