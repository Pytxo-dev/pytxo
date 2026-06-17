#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const webRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const chromaSrc = path.resolve(webRoot, "../../packages/chroma");
const chromaVendor = path.join(webRoot, "vendor", "chroma");

if (!fs.existsSync(chromaSrc)) {
  if (fs.existsSync(chromaVendor)) {
    process.exit(0);
  }
  console.warn("ensure-chroma: packages/chroma missing and no vendor copy");
  process.exit(0);
}

fs.mkdirSync(path.dirname(chromaVendor), { recursive: true });
fs.cpSync(chromaSrc, chromaVendor, { recursive: true });
console.log("ensure-chroma: synced packages/chroma → apps/web/vendor/chroma");
