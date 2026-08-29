#!/usr/bin/env node
import { readdirSync, statSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const webRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const docsDir = path.join(webRoot, "content", "docs");
const MIN_MDX = 20;

function countMdx(dir) {
  let n = 0;
  for (const name of readdirSync(dir)) {
    const full = path.join(dir, name);
    if (statSync(full).isDirectory()) n += countMdx(full);
    else if (name.endsWith(".mdx")) n += 1;
  }
  return n;
}

const count = countMdx(docsDir);
if (count < MIN_MDX) {
  console.error(
    `assert-docs-source: found ${count} MDX files under content/docs (need >= ${MIN_MDX})`,
  );
  process.exit(1);
}
console.log(`assert-docs-source: ${count} MDX files under content/docs`);
