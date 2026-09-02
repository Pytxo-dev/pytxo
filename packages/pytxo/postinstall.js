"use strict";

const fs = require("node:fs");
const path = require("node:path");
const http = require("node:http");
const https = require("node:https");
const crypto = require("node:crypto");
const { spawnSync } = require("node:child_process");

const { getAssetName } = require("./lib/platform");

const REPO = process.env.PYTXO_REPO || "Pytxo-dev/pytxo-releases";
const VERSION = process.env.PYTXO_VERSION || require("./package.json").version;
const TAG = VERSION.startsWith("v") ? VERSION : `v${VERSION}`;
const NORMALIZED_VERSION = VERSION.replace(/^v/, "");
const MAX_DOWNLOAD_BYTES = 200 * 1024 * 1024;
const MAX_REDIRECTS = 8;

const vendorDir = process.env.PYTXO_VENDOR_DIR || path.join(__dirname, "vendor");
const destName = process.platform === "win32" ? "pytxo.exe" : "pytxo";
const dest = path.join(vendorDir, destName);

function downloadBuffer(url, redirects = 0) {
  return new Promise((resolve, reject) => {
    const transport = new URL(url).protocol === "https:" ? https : http;
    transport
      .get(url, (res) => {
        if (res.statusCode === 302 || res.statusCode === 301) {
          const loc = res.headers.location;
          if (!loc) return reject(new Error("Redirect without location"));
          if (redirects >= MAX_REDIRECTS) {
            res.resume();
            return reject(new Error(`Too many redirects for ${url}`));
          }
          res.resume();
          return downloadBuffer(new URL(loc, url).toString(), redirects + 1).then(resolve, reject);
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        }
        const chunks = [];
        let bytes = 0;
        res.on("data", (chunk) => {
          bytes += chunk.length;
          if (bytes > MAX_DOWNLOAD_BYTES) {
            res.destroy(new Error(`Download exceeds ${MAX_DOWNLOAD_BYTES} bytes`));
            return;
          }
          chunks.push(chunk);
        });
        res.on("end", () => resolve(Buffer.concat(chunks)));
        res.on("error", reject);
      })
      .on("error", reject);
  });
}

function expectedChecksum(checksumText, asset) {
  for (const line of checksumText.split(/\r?\n/)) {
    const match = line.match(/^([a-fA-F0-9]{64})\s+\*?(.+)$/);
    if (match && match[2].trim() === asset) return match[1].toLowerCase();
  }
  throw new Error(`SHA256SUMS.txt has no exact entry for ${asset}`);
}

function sha256(bytes) {
  return crypto.createHash("sha256").update(bytes).digest("hex");
}

function assertChecksum(bytes, expected, label) {
  const actual = sha256(bytes);
  if (actual !== expected.toLowerCase()) {
    throw new Error(`SHA-256 mismatch for ${label}: expected ${expected}, got ${actual}`);
  }
}

function assertBinaryVersion(binaryPath, expectedVersion = NORMALIZED_VERSION) {
  const result = spawnSync(binaryPath, ["--version"], {
    encoding: "utf8",
    windowsHide: true,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`${path.basename(binaryPath)} --version exited ${result.status}`);
  }
  const output = `${result.stdout || ""}\n${result.stderr || ""}`.trim();
  const match = output.match(/\bpytxo\s+v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)\b/i);
  if (!match || match[1] !== expectedVersion) {
    throw new Error(`Expected pytxo ${expectedVersion}, received ${JSON.stringify(output)}`);
  }
}

function installAtomically(tempPath, destination) {
  const backup = `${destination}.previous-${process.pid}`;
  let movedExisting = false;
  try {
    if (fs.existsSync(destination)) {
      fs.renameSync(destination, backup);
      movedExisting = true;
    }
    fs.renameSync(tempPath, destination);
    if (movedExisting) fs.rmSync(backup, { force: true });
  } catch (error) {
    if (!fs.existsSync(destination) && movedExisting && fs.existsSync(backup)) {
      fs.renameSync(backup, destination);
    }
    throw error;
  }
}

async function main() {
  if (process.env.PYTXO_SKIP_DOWNLOAD === "1") {
    console.log("pytxo: PYTXO_SKIP_DOWNLOAD=1, skipping binary download (development only)");
    return;
  }
  const asset = getAssetName();
  fs.mkdirSync(vendorDir, { recursive: true });
  const releaseBase = process.env.PYTXO_RELEASE_BASE_URL
    ? `${process.env.PYTXO_RELEASE_BASE_URL.replace(/\/$/, "")}/${TAG}`
    : `https://github.com/${REPO}/releases/download/${TAG}`;
  const checksumUrl = `${releaseBase}/SHA256SUMS.txt`;
  const url = `${releaseBase}/${asset}`;
  console.log(`pytxo: downloading ${asset} from ${TAG}…`);
  const checksumText = (await downloadBuffer(checksumUrl)).toString("utf8");
  const checksum = expectedChecksum(checksumText, asset);

  if (fs.existsSync(dest)) {
    const installed = fs.readFileSync(dest);
    if (sha256(installed) === checksum) {
      assertBinaryVersion(dest);
      return;
    }
  }

  const bytes = await downloadBuffer(url);
  assertChecksum(bytes, checksum, asset);
  const suffix = process.platform === "win32" ? ".exe" : "";
  const temp = path.join(
    vendorDir,
    `.pytxo-download-${process.pid}-${crypto.randomBytes(6).toString("hex")}${suffix}`,
  );
  try {
    fs.writeFileSync(temp, bytes, { flag: "wx", mode: 0o755 });
    if (process.platform !== "win32") {
      fs.chmodSync(temp, 0o755);
    }
    assertBinaryVersion(temp);
    installAtomically(temp, dest);
  } finally {
    fs.rmSync(temp, { force: true });
  }
}

if (require.main === module) {
  main().catch((error) => {
    console.error(`pytxo: install failed: ${error.message}`);
    process.exitCode = 1;
  });
}

module.exports = {
  assertChecksum,
  expectedChecksum,
  installAtomically,
  sha256,
};
