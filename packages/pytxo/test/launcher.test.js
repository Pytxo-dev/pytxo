"use strict";

// First run of bin/pytxo.js when install scripts never ran (pnpm 10 default,
// `npm i --ignore-scripts`): vendor/ is empty and the launcher must fetch and
// verify the binary itself, or refuse with a clear message.

const assert = require("node:assert/strict");
const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const http = require("node:http");
const os = require("node:os");
const path = require("node:path");
const test = require("node:test");

const { getAssetName } = require("../lib/platform");
const { sha256 } = require("../postinstall");

const launcher = path.resolve(__dirname, "..", "bin", "pytxo.js");
const exe = process.platform === "win32" ? "pytxo.exe" : "pytxo";
const asset = getAssetName();

// Something that answers `--version` like a release binary: the built CLI, or
// a shell stand-in where the OS can execute one.
function releaseBinary() {
  for (const profile of ["release", "debug"]) {
    const cli = path.resolve(__dirname, "..", "..", "..", "target", profile, exe);
    if (!fs.existsSync(cli)) continue;
    const version = spawnSync(cli, ["--version"], { encoding: "utf8" }).stdout.match(/pytxo\s+v?(\S+)/i)?.[1];
    if (version) return { bytes: fs.readFileSync(cli), version };
  }
  if (process.platform === "win32") return null;
  return { bytes: Buffer.from('#!/bin/sh\necho "pytxo 9.9.9"\n'), version: "9.9.9" };
}

async function serveRelease(t, files) {
  const hits = [];
  const server = http.createServer((request, response) => {
    hits.push(request.url);
    const body = files[request.url];
    if (body === undefined) return response.writeHead(404).end();
    response.writeHead(200).end(body);
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => new Promise((resolve) => server.close(resolve)));
  return { base: `http://127.0.0.1:${server.address().port}`, hits };
}

function tempVendor(t) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), "pytxo-launcher-"));
  t.after(() => fs.rmSync(dir, { recursive: true, force: true }));
  return dir;
}

function runLauncher(env) {
  const childEnv = { ...process.env, ...env };
  delete childEnv.PYTXO_BINARY;
  delete childEnv.PYTXO_SKIP_DOWNLOAD;
  return new Promise((resolve) => {
    const child = spawn(process.execPath, [launcher, "--version"], {
      env: childEnv,
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (chunk) => (stdout += chunk));
    child.stderr.on("data", (chunk) => (stderr += chunk));
    child.on("close", (code) => resolve({ code, stdout, stderr }));
  });
}

test("first run downloads, verifies and runs the binary, then reuses it", async (t) => {
  const release = releaseBinary();
  if (!release) return t.skip("build pytxo-cli to run the Windows first-run download test");
  const { bytes, version } = release;
  const { base, hits } = await serveRelease(t, {
    [`/v${version}/${asset}`]: bytes,
    [`/v${version}/SHA256SUMS.txt`]: `${sha256(bytes)}  ${asset}\n`,
  });
  const vendor = tempVendor(t);
  const env = { PYTXO_RELEASE_BASE_URL: base, PYTXO_VENDOR_DIR: vendor, PYTXO_VERSION: version };

  const first = await runLauncher(env);
  assert.equal(first.code, 0, first.stderr);
  assert.match(first.stdout, new RegExp(`pytxo v?${version.replaceAll(".", "\\.")}`));
  assert.match(first.stderr, /checking it against SHA256SUMS\.txt/);
  assert.equal(sha256(fs.readFileSync(path.join(vendor, exe))), sha256(bytes));

  const requests = hits.length;
  const second = await runLauncher(env);
  assert.equal(second.code, 0, second.stderr);
  assert.equal(second.stderr, "");
  assert.equal(hits.length, requests, "second run must not download again");
});

test("offline first run fails with a clear error and installs nothing", async (t) => {
  const closed = http.createServer();
  await new Promise((resolve) => closed.listen(0, "127.0.0.1", resolve));
  const { port } = closed.address();
  await new Promise((resolve) => closed.close(resolve));
  const vendor = tempVendor(t);

  const result = await runLauncher({
    PYTXO_RELEASE_BASE_URL: `http://127.0.0.1:${port}`,
    PYTXO_VENDOR_DIR: vendor,
    PYTXO_VERSION: "1.3.1",
  });
  assert.equal(result.code, 1);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /could not install the pytxo binary: could not download .*SHA256SUMS\.txt/);
  assert.match(result.stderr, /nothing was installed/);
  assert.deepEqual(fs.readdirSync(vendor), []);
});

test("checksum mismatch refuses to install", async (t) => {
  const { base } = await serveRelease(t, {
    [`/v1.3.1/${asset}`]: Buffer.from("tampered bytes"),
    "/v1.3.1/SHA256SUMS.txt": `${sha256(Buffer.from("release bytes"))}  ${asset}\n`,
  });
  const vendor = tempVendor(t);

  const result = await runLauncher({ PYTXO_RELEASE_BASE_URL: base, PYTXO_VENDOR_DIR: vendor, PYTXO_VERSION: "1.3.1" });
  assert.equal(result.code, 1);
  assert.match(result.stderr, /SHA-256 mismatch/);
  assert.deepEqual(fs.readdirSync(vendor), []);
});
