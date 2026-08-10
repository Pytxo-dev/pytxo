import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

async function jsonVersion(file) {
  const value = JSON.parse(await readFile(file, "utf8")).version;
  if (typeof value !== "string" || !value) throw new Error(`Missing version in ${file}`);
  return value;
}

export async function verifyReleaseVersion(root, expected) {
  const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
  const rust = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!rust) throw new Error("Missing workspace package version in Cargo.toml");

  const versions = {
    rust,
    desktop: await jsonVersion(path.join(root, "apps", "desktop", "package.json")),
    tauri: await jsonVersion(path.join(root, "apps", "desktop", "src-tauri", "tauri.conf.json")),
    demo: await jsonVersion(path.join(root, "apps", "demo-video", "package.json")),
    npm: await jsonVersion(path.join(root, "packages", "pytxo", "package.json")),
  };
  expected ??= versions.rust;
  const mismatches = Object.entries(versions)
    .filter(([, version]) => version !== expected)
    .map(([name, version]) => `${name}=${version}`);
  if (mismatches.length) {
    throw new Error(`Release version must be ${expected}; found ${mismatches.join(", ")}`);
  }
  return versions;
}

const invoked = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (invoked) {
  const expected = process.argv[2];
  const root = path.resolve(import.meta.dirname, "..", "..");
  const versions = await verifyReleaseVersion(root, expected);
  console.log(`Verified Pytxo ${versions.rust}: ${Object.keys(versions).join(", ")}`);
}
