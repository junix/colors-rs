//! Human-oriented terminal swatches and PNG palette rendering.

use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

use chromap::{composite_over, Color, CompositeSpace, Rgb8};
use clap::ValueEnum;

const MAX_PNG_COLORS: usize = 256;
const GRID_COLUMNS: usize = 8;
const CELL_WIDTH: usize = 104;
const CELL_HEIGHT: usize = 80;
const CELL_PADDING: usize = 4;
const CHECKER_SIZE: usize = 8;
const CANVAS: [u8; 3] = [32, 36, 43];
const CHECKER_LIGHT: [u8; 3] = [238, 238, 238];
const CHECKER_DARK: [u8; 3] = [190, 190, 190];

/// Controls whether human-readable output contains ANSI color.
#[derive(Clone, Copy, Debug, ValueEnum)]
pub(crate) enum ColorPolicy {
    /// Use color only when stdout is an interactive, color-capable terminal.
    Auto,
    /// Emit ANSI color even when stdout is redirected.
    Always,
    /// Never emit ANSI color.
    Never,
}

#[derive(Clone, Copy, Debug)]
enum AnsiMode {
    TrueColor,
    Ansi256,
}

/// Rendering decisions for human-readable terminal output.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TerminalStyle {
    ansi: Option<AnsiMode>,
}

impl TerminalStyle {
    pub(crate) fn detect(policy: ColorPolicy, plain: bool, json: bool) -> Self {
        let enabled = should_enable_ansi(
            policy,
            plain,
            json,
            io::stdout().is_terminal(),
            env::var_os("NO_COLOR").is_some(),
            env::var("TERM").is_ok_and(|term| term == "dumb"),
        );

        Self {
            ansi: enabled.then(detect_ansi_mode),
        }
    }

    pub(crate) const fn has_swatches(self) -> bool {
        self.ansi.is_some()
    }

    pub(crate) fn decorate(self, color: Color, text: &str) -> String {
        match self.swatch(color) {
            Some(swatch) => format!("{swatch} {text}"),
            None => text.to_owned(),
        }
    }

    pub(crate) fn swatch(self, color: Color) -> Option<String> {
        let mode = self.ansi?;
        if color.is_opaque() {
            Some(paint_background(mode, color.to_rgb8(), "    "))
        } else {
            let dark = composite_over(color, Color::from_rgb8(32, 32, 32), CompositeSpace::Srgb);
            let light =
                composite_over(color, Color::from_rgb8(238, 238, 238), CompositeSpace::Srgb);
            Some(format!(
                "{}{}",
                paint_background(mode, dark.to_rgb8(), "  "),
                paint_background(mode, light.to_rgb8(), "  ")
            ))
        }
    }
}

const fn should_enable_ansi(
    policy: ColorPolicy,
    plain: bool,
    json: bool,
    stdout_is_terminal: bool,
    no_color: bool,
    dumb_terminal: bool,
) -> bool {
    if plain || json {
        return false;
    }
    match policy {
        ColorPolicy::Always => true,
        ColorPolicy::Never => false,
        ColorPolicy::Auto => stdout_is_terminal && !no_color && !dumb_terminal,
    }
}

/// Result metadata for an optional PNG artifact.
#[derive(Debug)]
pub(crate) struct PngReport {
    pub(crate) path: String,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) written: bool,
    pub(crate) stdout: bool,
}

/// Encodes and optionally writes a palette PNG.
pub(crate) fn output_png(
    path: &Path,
    colors: &[Color],
    dry_run: bool,
    force: bool,
) -> Result<PngReport, Box<dyn Error>> {
    let (data, width, height) = encode_png(colors)?;
    let stdout = path == Path::new("-");

    if !stdout {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if !parent.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("PNG parent directory does not exist: {}", parent.display()),
            )
            .into());
        }
        if path.exists() && !force {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "PNG output already exists: {}; pass --force to replace it",
                    path.display()
                ),
            )
            .into());
        }
    }

    if !dry_run {
        if stdout {
            let mut output = io::stdout().lock();
            output.write_all(&data)?;
            output.flush()?;
        } else {
            let mut options = OpenOptions::new();
            options.write(true);
            if force {
                options.create(true).truncate(true);
            } else {
                options.create_new(true);
            }
            let mut output = options.open(path)?;
            output.write_all(&data)?;
            output.flush()?;
        }
    }

    Ok(PngReport {
        path: path.to_string_lossy().into_owned(),
        width,
        height,
        written: !dry_run,
        stdout,
    })
}

fn detect_ansi_mode() -> AnsiMode {
    let colorterm = env::var("COLORTERM")
        .unwrap_or_default()
        .to_ascii_lowercase();
    let term = env::var("TERM").unwrap_or_default().to_ascii_lowercase();
    if colorterm.contains("truecolor")
        || colorterm.contains("24bit")
        || term.contains("truecolor")
        || term.contains("direct")
    {
        AnsiMode::TrueColor
    } else {
        AnsiMode::Ansi256
    }
}

fn paint_background(mode: AnsiMode, rgb: Rgb8, content: &str) -> String {
    match mode {
        AnsiMode::TrueColor => format!(
            "\u{1b}[48;2;{};{};{}m{content}\u{1b}[0m",
            rgb.r, rgb.g, rgb.b
        ),
        AnsiMode::Ansi256 => format!("\u{1b}[48;5;{}m{content}\u{1b}[0m", nearest_ansi_256(rgb)),
    }
}

fn nearest_ansi_256(rgb: Rgb8) -> u8 {
    const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];
    let mut best_code = 16;
    let mut best_distance = u32::MAX;

    for (red_index, red) in LEVELS.iter().copied().enumerate() {
        for (green_index, green) in LEVELS.iter().copied().enumerate() {
            for (blue_index, blue) in LEVELS.iter().copied().enumerate() {
                let code = 16 + 36 * red_index + 6 * green_index + blue_index;
                let distance = color_distance(rgb, [red, green, blue]);
                if distance < best_distance {
                    best_code = code as u8;
                    best_distance = distance;
                }
            }
        }
    }

    for index in 0..24 {
        let level = 8 + index * 10;
        let distance = color_distance(rgb, [level, level, level]);
        if distance < best_distance {
            best_code = 232 + index;
            best_distance = distance;
        }
    }

    best_code
}

fn color_distance(rgb: Rgb8, candidate: [u8; 3]) -> u32 {
    let red = i32::from(rgb.r) - i32::from(candidate[0]);
    let green = i32::from(rgb.g) - i32::from(candidate[1]);
    let blue = i32::from(rgb.b) - i32::from(candidate[2]);
    (red * red + green * green + blue * blue) as u32
}

fn encode_png(colors: &[Color]) -> Result<(Vec<u8>, u32, u32), Box<dyn Error>> {
    if colors.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot render an empty palette",
        )
        .into());
    }
    if colors.len() > MAX_PNG_COLORS {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("PNG output supports at most {MAX_PNG_COLORS} colors"),
        )
        .into());
    }

    let rows = colors.len().div_ceil(GRID_COLUMNS);
    let columns = colors.len().div_ceil(rows);
    let width = columns * CELL_WIDTH;
    let height = rows * CELL_HEIGHT;
    let mut pixels = vec![0; width * height * 3];

    for y in 0..height {
        for x in 0..width {
            let cell_row = y / CELL_HEIGHT;
            let row_start = cell_row * columns;
            let row_colors = (colors.len() - row_start).min(columns);
            let row_width = row_colors * CELL_WIDTH;
            let row_offset = (width - row_width) / 2;
            let in_row = (row_offset..row_offset + row_width).contains(&x);
            let local_x = if in_row {
                (x - row_offset) % CELL_WIDTH
            } else {
                0
            };
            let local_y = y % CELL_HEIGHT;
            let inside = in_row
                && (CELL_PADDING..CELL_WIDTH - CELL_PADDING).contains(&local_x)
                && (CELL_PADDING..CELL_HEIGHT - CELL_PADDING).contains(&local_y);
            let color_index = row_start + (x.saturating_sub(row_offset) / CELL_WIDTH);

            let pixel = if inside && color_index < colors.len() {
                let checker = if ((local_x - CELL_PADDING) / CHECKER_SIZE
                    + (local_y - CELL_PADDING) / CHECKER_SIZE)
                    % 2
                    == 0
                {
                    CHECKER_LIGHT
                } else {
                    CHECKER_DARK
                };
                composite_pixel(colors[color_index], checker)
            } else {
                CANVAS
            };
            let offset = (y * width + x) * 3;
            pixels[offset..offset + 3].copy_from_slice(&pixel);
        }
    }

    let width = width as u32;
    let height = height as u32;
    let mut data = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut data, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&pixels)?;
    }
    Ok((data, width, height))
}

fn composite_pixel(color: Color, background: [u8; 3]) -> [u8; 3] {
    let alpha = color.alpha();
    [
        blend_channel(color.red(), background[0], alpha),
        blend_channel(color.green(), background[1], alpha),
        blend_channel(color.blue(), background[2], alpha),
    ]
}

fn blend_channel(foreground: f64, background: u8, alpha: f64) -> u8 {
    (foreground.mul_add(alpha, f64::from(background) / 255.0 * (1.0 - alpha)) * 255.0)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::{should_enable_ansi, ColorPolicy};

    #[test]
    fn automatic_color_requires_a_capable_tty() {
        assert!(should_enable_ansi(
            ColorPolicy::Auto,
            false,
            false,
            true,
            false,
            false,
        ));
        assert!(!should_enable_ansi(
            ColorPolicy::Auto,
            false,
            false,
            false,
            false,
            false,
        ));
        assert!(!should_enable_ansi(
            ColorPolicy::Auto,
            false,
            false,
            true,
            true,
            false,
        ));
        assert!(!should_enable_ansi(
            ColorPolicy::Auto,
            false,
            false,
            true,
            false,
            true,
        ));
    }

    #[test]
    fn plain_and_json_override_forced_color() {
        assert!(!should_enable_ansi(
            ColorPolicy::Always,
            true,
            false,
            true,
            false,
            false,
        ));
        assert!(!should_enable_ansi(
            ColorPolicy::Always,
            false,
            true,
            true,
            false,
            false,
        ));
    }
}
