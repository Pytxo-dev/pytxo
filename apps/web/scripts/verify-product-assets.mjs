import { createHash } from "node:crypto";
import { access, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const WEB_ROOT = fileURLToPath(new URL("../", import.meta.url));
const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));
const DOCS_CAPTURE_DIR = path.join(REPO_ROOT, "docs", "_attachments", "desktop-2");
const WEB_CAPTURE_DIR = path.join(WEB_ROOT, "public", "product");

const ROUTES = [
  "operations",
  "workspaces",
  "runs",
  "flow",
  "approvals",
  "integrations",
  "settings",
  "topology-focus",
  "run-review",
];
const VIEWPORTS = [
  { slug: "1600x1000", width: 1600, height: 1000 },
  { slug: "1280x800", width: 1280, height: 800 },
  { slug: "960x640", width: 960, height: 640 },
];
const MARKETING_ROUTES = ["operations", "flow", "approvals", "integrations"];
const MARKETING_VIEWPORTS = VIEWPORTS.filter(({ slug }) => slug !== "1280x800");
const PNG_SIGNATURE = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);

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

const marketing = await inspectSet(
  MARKETING_ROUTES.flatMap((route) =>
    MARKETING_VIEWPORTS.map((viewport) => ({
      ...viewport,
      route,
      file: path.join(WEB_CAPTURE_DIR, `${route}-${viewport.slug}.png`),
    })),
  ),
  "Marketing product set",
);

let docs = [];
let docsAvailable = true;
try {
  await access(DOCS_CAPTURE_DIR);
} catch (error) {
  if (error?.code !== "ENOENT") {
    throw error;
  }
  docsAvailable = false;
}

if (docsAvailable) {
  docs = await inspectSet(
    ROUTES.flatMap((route) =>
      VIEWPORTS.map((viewport) => ({
        ...viewport,
        route,
        file: path.join(DOCS_CAPTURE_DIR, `${route}-${viewport.slug}.png`),
      })),
    ),
    "Desktop reference set",
  );

  for (const asset of marketing) {
    const source = docs.find(
      ({ route, slug }) => route === asset.route && slug === asset.slug,
    );
    if (!source || source.hash !== asset.hash) {
      throw new Error(`${asset.file} does not match its current Desktop reference capture`);
    }
  }
}

console.log(
  docs.length > 0
    ? `Verified ${docs.length} Desktop references and ${marketing.length} marketing product captures: correct dimensions, distinct content, and matching sources.`
    : `Verified ${marketing.length} deployment product captures: correct dimensions and distinct content. Canonical Desktop source parity runs from the monorepo checkout.`,
);
