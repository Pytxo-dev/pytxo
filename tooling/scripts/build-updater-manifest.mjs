#!/usr/bin/env node
// Builds the Tauri updater `latest.json` manifest from signed desktop
// artifacts produced by the `desktop-release` build matrix.
//
// Runs as a single writer in the release-mirroring job (never inside the
// per-platform build matrix) specifically to avoid the read-merge-upload
// race that hits `tauri-action`'s own `latest.json` uploader when multiple
// matrix legs try to patch the same release asset concurrently.
//
// Usage: node build-updater-manifest.mjs <sourceDir> <outDir>
//   sourceDir: directory containing one subdirectory per desktop artifact
//              (e.g. sourceDir/pytxo-desktop-windows-x64/**), as produced by
//              `actions/download-artifact` with a `pattern:` matching more
//              than one artifact and `merge-multiple` left at its default.
//   outDir:    directory to copy the chosen signed artifacts + latest.json
//              into (uploaded to the public release alongside the plain
//              installers).
//
// Env vars:
//   TAG_NAME:        release tag, e.g. "v0.5.0"
//   HAS_SIGNING_KEY: "true" if TAURI_SIGNING_PRIVATE_KEY was configured for
//                     this run; used only to decide whether missing/partial
//                     updater coverage should fail the step.

import fs from "node:fs";
import path from "node:path";

const PREFIX_TO_PLATFORM = {
  "pytxo-desktop-linux-x64": "linux-x86_64",
  "pytxo-desktop-darwin-arm64": "darwin-aarch64",
  "pytxo-desktop-darwin-x64": "darwin-x86_64",
  "pytxo-desktop-windows-x64": "windows-x86_64",
};

// When a platform produces more than one signed bundle (e.g. Windows NSIS
// *and* MSI when `bundle.targets: "all"`), prefer the format we publish as
// the plain installer so the updater and the manual download agree.
const PREFERRED_EXTENSION = {
  "linux-x86_64": ".appimage",
  // Tauri updater consumes the signed .app.tar.gz (or .tar.gz), not the .dmg installer.
  "darwin-aarch64": ".app.tar.gz",
  "darwin-x86_64": ".app.tar.gz",
  "windows-x86_64": ".msi",
};

const FALLBACK_EXTENSIONS = {
  "linux-x86_64": [".AppImage", ".appimage"],
  "darwin-aarch64": [".app.tar.gz", ".tar.gz"],
  "darwin-x86_64": [".app.tar.gz", ".tar.gz"],
  "windows-x86_64": [".msi", ".exe"],
};

function walk(dir) {
  const out = [];
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...walk(full));
    else out.push(full);
  }
  return out;
}

function main() {
  const [sourceDir, outDir] = process.argv.slice(2);
  if (!sourceDir || !outDir) {
    console.error("usage: build-updater-manifest.mjs <sourceDir> <outDir>");
    process.exit(1);
  }

  const tag = process.env.TAG_NAME;
  if (!tag) {
    console.error("TAG_NAME env var is required");
    process.exit(1);
  }
  const version = tag.replace(/^v/, "");
  const hasKey = process.env.HAS_SIGNING_KEY === "true";

  fs.mkdirSync(outDir, { recursive: true });

  const platforms = {};
  for (const [prefix, platformKey] of Object.entries(PREFIX_TO_PLATFORM)) {
    const files = walk(path.join(sourceDir, prefix));
    const candidates = files
      .filter((f) => f.endsWith(".sig"))
      .map((sigPath) => sigPath.slice(0, -".sig".length))
      .filter((artifactPath) => fs.existsSync(artifactPath));

    if (candidates.length === 0) {
      console.log(`updater: no signed artifact found for ${platformKey} (${prefix})`);
      continue;
    }

    const preferred = PREFERRED_EXTENSION[platformKey];
    const fallbacks = FALLBACK_EXTENSIONS[platformKey] ?? [];
    const lower = (p) => p.toLowerCase();
    const chosen =
      candidates.find((p) => lower(p).endsWith(preferred)) ??
      fallbacks
        .map((ext) => candidates.find((p) => lower(p).endsWith(ext.toLowerCase())))
        .find(Boolean) ??
      candidates[0];
    if (candidates.length > 1) {
      console.log(
        `updater: ${platformKey} had ${candidates.length} signed candidates, chose ${path.basename(chosen)}`,
      );
    }

    const signature = fs.readFileSync(`${chosen}.sig`, "utf8").trim();
    const safeName = `${platformKey}-${path.basename(chosen)}`.replace(/\s+/g, "-");
    fs.copyFileSync(chosen, path.join(outDir, safeName));
    platforms[platformKey] = {
      signature,
      url: `https://github.com/Pytxo-dev/pytxo-releases/releases/download/${tag}/${encodeURIComponent(safeName)}`,
    };
    console.log(`updater: staged ${platformKey} -> ${safeName}`);
  }

  const expected = Object.keys(PREFIX_TO_PLATFORM).length;
  const found = Object.keys(platforms).length;

  if (found === 0) {
    if (hasKey) {
      console.log(
        "::warning::TAURI_SIGNING_PRIVATE_KEY is set but no signed updater artifacts were found — updater manifest skipped for this release.",
      );
    } else {
      console.log("No signing key configured — skipping updater manifest for this release (expected).");
    }
    return;
  }

  const manifest = {
    version,
    notes: `See https://github.com/Pytxo-dev/pytxo/blob/main/distribution/release-notes/v${version}.md`,
    pub_date: new Date().toISOString(),
    platforms,
  };
  fs.writeFileSync(path.join(outDir, "latest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
  console.log(`updater: wrote ${outDir}/latest.json with ${found}/${expected} platforms`);

  if (hasKey && found < expected) {
    console.log(
      `::error::TAURI_SIGNING_PRIVATE_KEY is set but only ${found}/${expected} platforms produced signed updater artifacts.`,
    );
    process.exit(1);
  }
}

main();
