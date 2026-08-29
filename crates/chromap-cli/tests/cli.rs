//! Binary-boundary tests for terminal swatches and PNG output.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn chromap(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_chromap"))
        .args(args)
        .output()
        .expect("run chromap")
}

fn temporary_png(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "chromap-{label}-{}-{nonce}.png",
        std::process::id()
    ))
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("UTF-8 stdout")
}

fn stderr_text(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("UTF-8 stderr")
}

#[test]
fn plain_palette_is_stable_text_without_ansi() {
    let output = chromap(&[
        "--plain",
        "palette",
        "#4f7cff",
        "--kind",
        "hue-wheel",
        "--count",
        "3",
    ]);
    assert!(output.status.success(), "{}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains('\u{1b}'));
}

#[test]
fn automatic_color_is_disabled_when_stdout_is_piped() {
    let output = Command::new(env!("CARGO_BIN_EXE_chromap"))
        .args(["convert", "#4f7cff"])
        .env("COLORTERM", "truecolor")
        .output()
        .expect("run chromap");
    assert!(output.status.success(), "{}", stderr_text(&output));
    assert_eq!(stdout_text(&output), "#4f7cff\n");
}

#[test]
fn forced_truecolor_adds_background_swatches() {
    let output = Command::new(env!("CARGO_BIN_EXE_chromap"))
        .args(["--color", "always", "palette", "#4f7cff", "--count", "2"])
        .env("COLORTERM", "truecolor")
        .env("TERM", "xterm-256color")
        .output()
        .expect("run chromap");
    assert!(output.status.success(), "{}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("\u{1b}[48;2;"));
    assert_eq!(stdout.matches("\u{1b}[0m").count(), 2);
}

#[test]
fn forced_color_falls_back_to_ansi_256() {
    let output = Command::new(env!("CARGO_BIN_EXE_chromap"))
        .args(["--color", "always", "convert", "#4f7cff"])
        .env_remove("COLORTERM")
        .env("TERM", "xterm-256color")
        .output()
        .expect("run chromap");
    assert!(output.status.success(), "{}", stderr_text(&output));
    let stdout = stdout_text(&output);
    assert!(stdout.contains("\u{1b}[48;5;"));
    assert!(!stdout.contains("\u{1b}[48;2;"));
}

#[test]
fn plain_overrides_forced_color() {
    let output = chromap(&["--plain", "--color", "always", "convert", "#4f7cff"]);
    assert!(output.status.success(), "{}", stderr_text(&output));
    assert_eq!(stdout_text(&output), "#4f7cff\n");
}

#[test]
fn json_remains_valid_and_ansi_free() {
    let output = chromap(&[
        "--json", "--color", "always", "palette", "#4f7cff", "--count", "2",
    ]);
    assert!(output.status.success(), "{}", stderr_text(&output));
    assert!(!output.stdout.contains(&0x1b));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(value["colors"].as_array().expect("colors array").len(), 2);
}

#[test]
fn alpha_color_uses_dark_and_light_terminal_samples() {
    let output = Command::new(env!("CARGO_BIN_EXE_chromap"))
        .args(["--color", "always", "convert", "#ff000080"])
        .env("COLORTERM", "truecolor")
        .output()
        .expect("run chromap");
    assert!(output.status.success(), "{}", stderr_text(&output));
    assert_eq!(stdout_text(&output).matches("\u{1b}[0m").count(), 2);
}

#[test]
fn palette_writes_decodable_png_layout() {
    let path = temporary_png("palette");
    let output = chromap(&[
        "--plain",
        "palette",
        "#4f7cff",
        "--kind",
        "hue-wheel",
        "--count",
        "9",
        "--png",
        path.to_str().expect("UTF-8 path"),
    ]);
    assert!(output.status.success(), "{}", stderr_text(&output));
    assert!(stderr_text(&output).contains("wrote PNG:"));

    let bytes = fs::read(&path).expect("read PNG");
    assert_eq!(&bytes[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&bytes[12..16], b"IHDR");
    assert_eq!(u32::from_be_bytes(bytes[16..20].try_into().unwrap()), 520);
    assert_eq!(u32::from_be_bytes(bytes[20..24].try_into().unwrap()), 160);
    fs::remove_file(path).expect("remove PNG");
}

#[test]
fn png_dry_run_reports_without_writing() {
    let path = temporary_png("dry-run");
    let output = chromap(&[
        "--json",
        "gradient",
        "red",
        "blue",
        "--png",
        path.to_str().expect("UTF-8 path"),
        "--dry-run",
    ]);
    assert!(output.status.success(), "{}", stderr_text(&output));
    assert!(!path.exists());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(value["png"]["written"], false);
    assert_eq!(value["png"]["width"], 728);
    assert_eq!(value["png"]["height"], 80);
}

#[test]
fn png_refuses_to_replace_without_force() {
    let path = temporary_png("exists");
    fs::write(&path, b"keep me").expect("seed output");
    let output = chromap(&[
        "--plain",
        "palette",
        "red",
        "--png",
        path.to_str().expect("UTF-8 path"),
    ]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("pass --force to replace it"));
    assert_eq!(fs::read(&path).unwrap(), b"keep me");
    fs::remove_file(path).expect("remove seed file");
}

#[test]
fn png_can_be_the_only_stdout_payload() {
    let output = chromap(&[
        "--plain", "gradient", "red", "blue", "--steps", "3", "--png", "-",
    ]);
    assert!(output.status.success(), "{}", stderr_text(&output));
    assert_eq!(&output.stdout[..8], b"\x89PNG\r\n\x1a\n");
    assert!(stderr_text(&output).contains("wrote PNG: -"));
}

#[test]
fn json_rejects_binary_png_stdout() {
    let output = chromap(&["--json", "palette", "red", "--png", "-"]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("use --png PATH"));
}

#[test]
fn css_rejects_binary_png_stdout() {
    let output = chromap(&[
        "gradient",
        "red",
        "blue",
        "--css-prefix",
        "accent",
        "--png",
        "-",
    ]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("--css-prefix cannot be combined"));
}

#[test]
fn png_rejects_more_than_256_colors() {
    let path = temporary_png("too-many");
    let output = chromap(&[
        "--plain",
        "palette",
        "red",
        "--kind",
        "hue-wheel",
        "--count",
        "257",
        "--png",
        path.to_str().expect("UTF-8 path"),
        "--dry-run",
    ]);
    assert!(!output.status.success());
    assert!(stderr_text(&output).contains("at most 256 colors"));
    assert!(!path.exists());
}
