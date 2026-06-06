"use strict";

const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");

const { getAssetName } = require("./lib/platform");

const REPO = process.env.PYTXO_REPO || "Pytxo-dev/pytxo";
const VERSION = process.env.PYTXO_VERSION || require("./package.json").version;
const TAG = VERSION.startsWith("v") ? VERSION : `v${VERSION}`;

const vendorDir = path.join(__dirname, "vendor");
const asset = getAssetName();
const destName = process.platform === "win32" ? "pytxo.exe" : "pytxo";
const dest = path.join(vendorDir, destName);

function download(url) {
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(dest);
    https
      .get(url, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          const loc = res.headers.location;
          if (!loc) return reject(new Error("Redirect without location"));
          res.resume();
          return download(loc).then(resolve, reject);
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        }
        res.pipe(file);
        file.on("finish", () => file.close(resolve));
      })
      .on("error", reject);
  });
}

async function main() {
  if (process.env.PYTXO_SKIP_DOWNLOAD === "1") {
    console.log("pytxo: PYTXO_SKIP_DOWNLOAD=1, skipping binary download");
    return;
  }
  if (fs.existsSync(dest)) {
    return;
  }
  fs.mkdirSync(vendorDir, { recursive: true });
  const url = `https://github.com/${REPO}/releases/download/${TAG}/${asset}`;
  console.log(`pytxo: downloading ${asset} from ${TAG}…`);
  try {
    await download(url);
    if (process.platform !== "win32") {
      fs.chmodSync(dest, 0o755);
    }
  } catch (err) {
    console.warn(
      `pytxo: could not download release binary (${err.message}).\n` +
        `  Install from source: cargo install --path crates/pytxo-cli\n` +
        `  Or run: curl -fsSL https://raw.githubusercontent.com/${REPO}/main/tooling/scripts/install.sh | bash`,
    );
  }
}

main();
