import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

if (process.platform !== "win32") {
  console.error("The Windows MSI build requires Windows and its native build tools.");
  process.exit(1);
}

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const result = spawnSync("cargo", [
  "tauri", "build",
  "--target", "x86_64-pc-windows-msvc",
  "--bundles", "msi",
  "--features", "voice-whisper",
  // These are Cargo arguments, not Tauri configuration options. An explicit
  // target keeps the distribution CRT flags out of host build scripts/macros.
  "--", "--locked", "--config", path.join(root, ".cargo/windows-msvc.toml"),
], { cwd: path.join(root, "apps/desktop"), stdio: "inherit", windowsHide: true });
if (result.error) console.error(result.error.message);
if (result.status !== 0) process.exit(result.status ?? 1);
// Cargo honors CARGO_TARGET_DIR; verify the executable it actually produced.
const targetDir = process.env.CARGO_TARGET_DIR
  ? path.resolve(process.env.CARGO_TARGET_DIR)
  : path.join(root, "target");
const assets = spawnSync(process.execPath, [
  path.join(root, "tooling/scripts/verify-desktop-embedded-assets.mjs"),
  path.join(targetDir, "x86_64-pc-windows-msvc/release/pytxo-desktop.exe"),
  path.join(root, "apps/desktop/dist"),
], { stdio: "inherit", windowsHide: true });
if (assets.error) console.error(assets.error.message);
process.exit(assets.status ?? 1);
