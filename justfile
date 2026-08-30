set shell := ["bash", "-euo", "pipefail", "-c"]

os_name := if os() == "macos" { "macos" } else { "linux" }
arch_name := if arch() == "aarch64" { "arm64" } else { "x86" }
default_install_bin := home_directory() / "sync" / (os_name + "-" + arch_name + "-bin")
install_bin := env("SYNC_BIN_DIR", default_install_bin)
target_dir := env("CARGO_TARGET_DIR", justfile_directory() / "target")

# Show the available project commands.
@default:
    just --list

# Format all Rust sources.
fmt:
    cargo fmt --all

# Verify Rust formatting without changing files.
fmt-check:
    cargo fmt --all -- --check

# Type-check the complete workspace.
check:
    cargo check --workspace --all-targets --all-features

# Run Clippy with the same strictness as CI.
clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all workspace tests.
test:
    cargo test --workspace --all-features

# Build documentation and reject rustdoc warnings.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

# Build the CLI in release mode.
build:
    cargo build --release --locked --package chromap-cli

# Verify that the npm package contains all Rust sources, examples, and the CLI launcher.
npm-check:
    npm run check:package

# Run the CLI from source, for example: `just run --version`.
run *args:
    cargo run --quiet --package chromap-cli --bin chromap -- {{args}}

# Install to ~/sync/<os>-<arch>-bin. Override with SYNC_BIN_DIR.
install: build
    mkdir -p "{{ install_bin }}"
    cp "{{ target_dir }}/release/chromap" "{{ install_bin }}/chromap"
    chmod +x "{{ install_bin }}/chromap"
    echo "Installed {{ install_bin }}/chromap"

# Run every CI validation gate plus a real CLI smoke test.
ci: fmt-check clippy test doc npm-check
    cargo run --quiet --package chromap-cli --bin chromap -- --version
