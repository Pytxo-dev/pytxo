#!/usr/bin/env node
/**
 * Set Vercel production env vars from a .env file (non-interactive).
 * Usage: node set-vercel-env.mjs <projectDir> <envFile> KEY1 KEY2 ...
 */
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const projectDir = process.argv[2];
const envFile = process.argv[3];
const keys = process.argv.slice(4);
if (!projectDir || !envFile || keys.length === 0) {
  console.error("Usage: node set-vercel-env.mjs <projectDir> <envFile> KEY ...");
  process.exit(1);
}

const map = Object.fromEntries(
  fs
    .readFileSync(envFile, "utf8")
    .split(/\r?\n/)
    .map((raw) => raw.replace(/\r$/, ""))
    .map((line) => {
      const m = line.match(/^([A-Z0-9_]+)=(.*)$/);
      if (!m) return null;
      let v = m[2];
      if (v.startsWith('"') && v.endsWith('"')) v = v.slice(1, -1);
      return [m[1].trim(), v.trim()];
    })
    .filter(Boolean),
);

for (const key of keys) {
  const value = map[key];
  if (!value) {
    console.warn(`skip ${key}: missing in ${envFile}`);
    continue;
  }
  try {
    execFileSync(
      "npx",
      ["vercel", "env", "rm", key, "production", "--yes"],
      { cwd: projectDir, stdio: "ignore", shell: true },
    );
  } catch {
    /* may not exist */
  }
  execFileSync("npx", ["vercel", "env", "add", key, "production"], {
    cwd: projectDir,
    input: value,
    stdio: ["pipe", "inherit", "inherit"],
    shell: true,
  });
  console.log(`set ${key} (${value.length} chars)`);
}
