"use strict";

const assert = require("node:assert/strict");
const { spawn } = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const { getAssetName } = require("../lib/platform");
const {
  assertChecksum,
  expectedChecksum,
  installAtomically,
  sha256,
} = require("../postinstall");

test("release platform mapping is exact and rejects undeclared targets", () => {
  assert.equal(getAssetName("win32", "x64"), "pytxo-windows-x64.exe");
  assert.equal(getAssetName("darwin", "arm64"), "pytxo-darwin-arm64");
  assert.equal(getAssetName("linux", "x64"), "pytxo-linux-x64");
  assert.throws(() => getAssetName("win32", "arm64"), /Unsupported Pytxo release platform/);
  assert.throws(() => getAssetName("linux", "ia32"), /Unsupported Pytxo release platform/);
});

test("checksum manifest requires an exact asset entry", () => {
  const bytes = Buffer.from("release bytes");
  const digest = sha256(bytes);
  const manifest = `${digest}  pytxo-linux-x64\n${"0".repeat(64)}  pytxo-linux-x64-extra\n`;
  assert.equal(expectedChecksum(manifest, "pytxo-linux-x64"), digest);
  assert.throws(() => expectedChecksum(manifest, "pytxo-linux-arm64"), /no exact entry/);
  assert.doesNotThrow(() => assertChecksum(bytes, digest, "fixture"));
  assert.throws(() => assertChecksum(Buffer.from("changed"), digest, "fixture"), /mismatch/);
});

test("atomic install replaces a destination and leaves no backup", () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "pytxo-postinstall-"));
  const destination = path.join(dir, "pytxo");
  const temp = path.join(dir, ".download");
  try {
    fs.writeFileSync(destination, "old");
    fs.writeFileSync(temp, "new");
    installAtomically(temp, destination);
    assert.equal(fs.readFileSync(destination, "utf8"), "new");
    assert.equal(fs.existsSync(temp), false);
    assert.deepEqual(fs.readdirSync(dir), ["pytxo"]);
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
});

function builtCli() {
  const executable = process.platform === "win32" ? "pytxo.exe" : "pytxo";
  for (const profile of ["release", "debug"]) {
    const candidate = path.resolve(__dirname, "..", "..", "..", "target", profile, executable);
    if (fs.existsSync(candidate)) return candidate;
  }
  return null;
}

function runInstaller(env) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, [path.resolve(__dirname, "..", "postinstall.js")], {
      env: { ...process.env, ...env },
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => (stdout += chunk));
    child.stderr.on("data", (chunk) => (stderr += chunk));
    child.on("close", (code) => resolve({ code, stdout, stderr }));
  });
}

test("postinstall verifies staged bytes and fails closed on checksum mismatch", async (t) => {
  const cli = builtCli();
  if (!cli) return t.skip("build pytxo-cli before the staged installer integration test");

  const bytes = fs.readFileSync(cli);
  const asset = getAssetName();
  let checksum = sha256(bytes);
  const server = http.createServer((request, response) => {
    if (request.url === `/v1.2.2/${asset}`) {
      response.writeHead(200, { "content-type": "application/octet-stream" });
      response.end(bytes);
      return;
    }
    if (request.url === "/v1.2.2/SHA256SUMS.txt") {
      response.writeHead(200, { "content-type": "text/plain" });
      response.end(`${checksum}  ${asset}\n`);
      return;
    }
    response.writeHead(404).end();
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  const base = `http://127.0.0.1:${address.port}`;
  const vendor = fs.mkdtempSync(path.join(os.tmpdir(), "pytxo-vendor-"));
  try {
    const installed = await runInstaller({
      PYTXO_RELEASE_BASE_URL: base,
      PYTXO_VENDOR_DIR: vendor,
      PYTXO_VERSION: "1.2.2",
    });
    assert.equal(installed.code, 0, installed.stderr);
    const destination = path.join(vendor, process.platform === "win32" ? "pytxo.exe" : "pytxo");
    assert.equal(sha256(fs.readFileSync(destination)), sha256(bytes));

    fs.rmSync(destination, { force: true });
    checksum = "0".repeat(64);
    const rejected = await runInstaller({
      PYTXO_RELEASE_BASE_URL: base,
      PYTXO_VENDOR_DIR: vendor,
      PYTXO_VERSION: "1.2.2",
    });
    assert.notEqual(rejected.code, 0);
    assert.match(rejected.stderr, /SHA-256 mismatch/);
    assert.equal(fs.existsSync(destination), false);
  } finally {
    await new Promise((resolve) => server.close(resolve));
    fs.rmSync(vendor, { recursive: true, force: true });
  }
});
