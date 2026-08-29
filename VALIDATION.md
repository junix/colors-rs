# Validation status

The initial artifact-generation environment completed:

- all Rust files lexed without lexer error;
- balanced delimiter scan excluding comments and strings;
- all TOML manifests parsed with Python `tomllib`;
- workspace/member/file-reference consistency checks;
- ZIP CRC verification.

Release validation must execute these commands in a Rust environment:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo doc --workspace --all-features --no-deps
npm test
```

The GitHub Actions workflow runs those commands and smoke-tests the npm CLI launcher.
