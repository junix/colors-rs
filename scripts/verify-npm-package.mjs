import { readdirSync } from "node:fs";
import { join, relative } from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));

function collectRustFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...collectRustFiles(path));
    if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(relative(root, path).replaceAll("\\", "/"));
    }
  }
  return files;
}

const packed = spawnSync(
  "npm",
  ["pack", "--dry-run", "--json", "--ignore-scripts"],
  { cwd: root, encoding: "utf8" },
);

if (packed.status !== 0) {
  process.stderr.write(packed.stderr);
  process.exit(packed.status ?? 1);
}

const [{ files }] = JSON.parse(packed.stdout);
const packedPaths = new Set(files.map(({ path }) => path));
const rustFiles = collectRustFiles(join(root, "crates"));
const missing = rustFiles.filter((path) => !packedPaths.has(path));
const examples = rustFiles.filter((path) => path.includes("/examples/"));
const requiredPackageFiles = [
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  "crates/chromap/Cargo.toml",
  "crates/chromap-cli/Cargo.toml",
  "bin/chromap.js",
];
const missingRequiredFiles = requiredPackageFiles.filter(
  (path) => !packedPaths.has(path),
);

if (rustFiles.length === 0) throw new Error("no Rust source files found");
if (examples.length === 0) throw new Error("no Rust examples found");
if (missing.length > 0) {
  throw new Error(`npm package is missing Rust files:\n${missing.join("\n")}`);
}
if (missingRequiredFiles.length > 0) {
  throw new Error(
    `npm package is missing required files:\n${missingRequiredFiles.join("\n")}`,
  );
}

console.log(
  `verified npm package: ${rustFiles.length} Rust files, ${examples.length} example(s), and the chromap CLI launcher`,
);
