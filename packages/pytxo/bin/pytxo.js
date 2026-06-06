#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const path = require("node:path");

const binName = process.platform === "win32" ? "pytxo.exe" : "pytxo";
const vendor = path.join(__dirname, "..", "vendor", binName);
const envBin = process.env.PYTXO_BINARY;

const candidates = [envBin, vendor].filter(Boolean);

for (const bin of candidates) {
  if (fs.existsSync(bin)) {
    const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
    if (result.error) {
      console.error(result.error.message);
      process.exit(1);
    }
    process.exit(result.status ?? 1);
  }
}

console.error(
  "pytxo binary not found. Re-run npm install or set PYTXO_BINARY to a local build.",
);
process.exit(1);
