#!/usr/bin/env node
"use strict";

const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");

const { dest, install } = require("../postinstall");

// Lets the CLI say "run npx pytxo again" when it was started that way.
const runAs = process.env.PYTXO_RUN_AS || (process.env.npm_command === "exec" ? "npx pytxo" : "pytxo");

function run(bin) {
  // The binary handles Ctrl+C itself: it removes its scratch checkout, a
  // second Ctrl+C quits it at once, and it exits 130. Waiting for it keeps the
  // terminal from coming back while it cleans up.
  for (const signal of ["SIGINT", "SIGTERM", "SIGHUP"]) process.on(signal, () => {});
  const result = spawnSync(bin, process.argv.slice(2), {
    stdio: "inherit",
    env: { ...process.env, PYTXO_RUN_AS: runAs },
  });
  if (result.error) {
    console.error(result.error.message);
    process.exit(1);
  }
  // Killed by a signal: exit as a shell reports it, 128 plus its number.
  process.exit(result.status ?? (result.signal ? 128 + os.constants.signals[result.signal] : 1));
}

async function main() {
  const envBin = process.env.PYTXO_BINARY;
  if (envBin && fs.existsSync(envBin)) run(envBin);
  if (fs.existsSync(dest)) run(dest);

  if (process.env.PYTXO_SKIP_DOWNLOAD === "1") {
    console.error("pytxo binary not found. Unset PYTXO_SKIP_DOWNLOAD or set PYTXO_BINARY to a local build.");
    process.exit(1);
  }

  // Install scripts were skipped (pnpm 10 blocks them by default, and so does
  // `npm i --ignore-scripts`): fetch and verify the binary now, once.
  try {
    await install();
  } catch (error) {
    // A concurrent first run may have installed it in the meantime.
    if (!fs.existsSync(dest)) {
      console.error(`pytxo: could not install the pytxo binary: ${error.message}`);
      console.error(
        "pytxo: nothing was installed. Check your network and run pytxo again, or set PYTXO_BINARY to a local build.",
      );
      process.exit(1);
    }
  }
  run(dest);
}

main();
