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
process.exit(result.status ?? 1);
