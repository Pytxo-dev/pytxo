import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

// Tauri embeds asset keys in the executable even when the payload is compressed.
// Vite hashes JS/CSS filenames by content. Check those identities so an incremental
// native build cannot silently package a previous frontend. This is not runtime QA.
const [executablePath, frontendDirectory] = process.argv.slice(2);
if (!executablePath || !frontendDirectory) {
  throw new Error("Usage: node verify-desktop-embedded-assets.mjs <executable> <frontend-dist>");
}
const assets = readdirSync(path.join(frontendDirectory, "assets"))
  .filter(name => /-[A-Za-z0-9_-]+\.(?:js|css)$/.test(name));
if (!assets.length) throw new Error("No hashed frontend JS/CSS assets found.");
const binary = readFileSync(executablePath);
const missing = assets.filter(name => !binary.includes(Buffer.from(`assets/${name}`)));
if (missing.length) {
  throw new Error(`Stale embedded frontend: executable lacks ${missing.join(", ")}`);
}
console.log(`Verified ${assets.length} current hashed JS/CSS asset identities in ${executablePath}.`);
