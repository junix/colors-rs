#!/usr/bin/env node

const { existsSync, mkdirSync, readFileSync } = require("node:fs");
const { homedir } = require("node:os");
const { dirname, join, resolve } = require("node:path");
const { spawnSync } = require("node:child_process");

const packageRoot = resolve(__dirname, "..");
const manifestPath = join(packageRoot, "Cargo.toml");
const packageMetadata = JSON.parse(
  readFileSync(join(packageRoot, "package.json"), "utf8"),
);

function defaultCacheRoot() {
  if (process.env.CHROMAP_CACHE_DIR) return process.env.CHROMAP_CACHE_DIR;
  if (process.env.XDG_CACHE_HOME) return join(process.env.XDG_CACHE_HOME, "chromap");
  if (process.platform === "win32" && process.env.LOCALAPPDATA) {
    return join(process.env.LOCALAPPDATA, "chromap");
  }
  if (process.platform === "darwin") return join(homedir(), "Library", "Caches", "chromap");
  return join(homedir(), ".cache", "chromap");
}

const targetDirectory = join(
  defaultCacheRoot(),
  `npm-v${packageMetadata.version}-${process.platform}-${process.arch}`,
  "target",
);
const executable = join(
  targetDirectory,
  "release",
  process.platform === "win32" ? "chromap.exe" : "chromap",
);

if (!existsSync(executable)) {
  mkdirSync(dirname(executable), { recursive: true });
  const build = spawnSync(
    "cargo",
    [
      "build",
      "--release",
      "--locked",
      "--package",
      "chromap-cli",
      "--manifest-path",
      manifestPath,
      "--target-dir",
      targetDirectory,
    ],
    { stdio: "inherit" },
  );

  if (build.error?.code === "ENOENT") {
    console.error(
      "chromap: Rust and Cargo are required for the first run. Install Rust 1.81 or newer from https://rustup.rs/.",
    );
    process.exit(127);
  }
  if (build.error) {
    console.error(`chromap: failed to start Cargo: ${build.error.message}`);
    process.exit(1);
  }
  if (build.status !== 0) process.exit(build.status ?? 1);
}

const result = spawnSync(executable, process.argv.slice(2), { stdio: "inherit" });
if (result.error) {
  console.error(`chromap: failed to start the CLI: ${result.error.message}`);
  process.exit(1);
}
process.exit(result.status ?? 1);
