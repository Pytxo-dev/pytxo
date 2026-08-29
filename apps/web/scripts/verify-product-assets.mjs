import { createHash } from "node:crypto";
import { access, readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  MARKETING_ROUTES,
  MARKETING_VIEWPORTS,
  REFERENCE_ROUTES,
  VIEWPORTS,
} from "./product-asset-manifest.mjs";

const WEB_ROOT = fileURLToPath(new URL("../", import.meta.url));
const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));
const DESKTOP_CAPTURE_DIR = path.join(
  REPO_ROOT,
  "apps",
  "desktop",
  "captures",
  "desktop-2",
);
const DOCS_CAPTURE_DIR = path.join(REPO_ROOT, "docs", "_attachments", "desktop-2");
const WEB_CAPTURE_DIR = path.join(WEB_ROOT, "public", "product");

const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

function entriesFor(directory, routes, viewports) {
  return routes.flatMap((route) =>
    viewports.map((viewport) => ({
      ...viewport,
      route,
      file: path.join(directory, `${route}-${viewport.slug}.png`),
    })),
  );
}

function inspectPng(buffer, file, expected) {
  if (buffer.byteLength < 40_000) {
    throw new Error(`${file} is unexpectedly small (${buffer.byteLength} bytes)`);
  }
  if (!buffer.subarray(0, 8).equals(PNG_SIGNATURE)) {
    throw new Error(`${file} is not a PNG`);
  }

  const width = buffer.readUInt32BE(16);
  const height = buffer.readUInt32BE(20);
  if (width !== expected.width || height !== expected.height) {
    throw new Error(
      `${file} is ${width}x${height}; expected ${expected.width}x${expected.height}`,
    );
  }

  return createHash("sha256").update(buffer).digest("hex");
}

async function inspectSet(entries, label) {
  const hashes = new Map();
  for (const entry of entries) {
    const buffer = await readFile(entry.file);
    const hash = inspectPng(buffer, entry.file, entry);
    const duplicate = hashes.get(hash);
    if (duplicate) {
      throw new Error(`${label} duplicates product evidence: ${duplicate} and ${entry.file}`);
    }
    hashes.set(hash, entry.file);
    entry.hash = hash;
  }
  return entries;
}

async function directoryExists(directory) {
  try {
    await access(directory);
    return true;
  } catch (error) {
    if (error?.code === "ENOENT") return false;
    throw error;
  }
}

async function requireDirectory(directory, label) {
  if (!(await directoryExists(directory))) {
    throw new Error(`Missing ${label}: ${directory}`);
  }
}

function assertParity(actual, source, label) {
  for (const asset of actual) {
    const reference = source.find(
      ({ route, slug }) => route === asset.route && slug === asset.slug,
    );
    if (!reference || reference.hash !== asset.hash) {
      throw new Error(`${asset.file} does not match ${label}`);
    }
  }
}

const marketingEntries = entriesFor(WEB_CAPTURE_DIR, MARKETING_ROUTES, MARKETING_VIEWPORTS);

// A stale capture of a route the app no longer has is a false product claim, so
// the published directory must contain exactly the manifest and nothing else.
const published = await readdir(WEB_CAPTURE_DIR);
const permitted = new Set(marketingEntries.map(({ file }) => path.basename(file)));
const unexpected = published.filter((name) => !permitted.has(name));
if (unexpected.length > 0) {
  throw new Error(
    `Unmanifested marketing captures in ${WEB_CAPTURE_DIR}: ${unexpected.join(", ")}. ` +
      "Run npm run sync:product-assets.",
  );
}

const marketing = await inspectSet(marketingEntries, "Marketing product set");

await requireDirectory(
  DESKTOP_CAPTURE_DIR,
  "canonical deterministic Desktop capture directory",
);
const desktop = await inspectSet(
  entriesFor(DESKTOP_CAPTURE_DIR, REFERENCE_ROUTES, VIEWPORTS),
  "Deterministic Desktop source set",
);

let docs = [];
if (await directoryExists(DOCS_CAPTURE_DIR)) {
  docs = await inspectSet(
    entriesFor(DOCS_CAPTURE_DIR, REFERENCE_ROUTES, VIEWPORTS),
    "Documentation reference set",
  );
}

if (docs.length > 0) {
  assertParity(docs, desktop, "its deterministic Desktop source capture");
}

assertParity(marketing, desktop, "its deterministic Desktop source capture");

console.log(
  `Verified ${desktop.length} Desktop source captures, ${docs.length} documentation references, and ${marketing.length} marketing captures: dimensions, distinct content, and source parity.`,
);
