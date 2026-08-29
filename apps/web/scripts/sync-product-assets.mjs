import { copyFile, mkdir, readdir, rm } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { MARKETING_ROUTES, MARKETING_VIEWPORTS } from "./product-asset-manifest.mjs";

/**
 * Marketing captures used to be copied out of the Desktop capture directory by
 * hand, which let the site keep serving screenshots of routes the app no longer
 * had. This mirrors them instead, and deletes anything not in the manifest so a
 * renamed route cannot leave a stale image behind.
 */
const WEB_ROOT = fileURLToPath(new URL("../", import.meta.url));
const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));
const SOURCE_DIR = path.join(REPO_ROOT, "apps", "desktop", "captures", "desktop-2");
const TARGET_DIR = path.join(WEB_ROOT, "public", "product");

const expected = new Set(
  MARKETING_ROUTES.flatMap((route) =>
    MARKETING_VIEWPORTS.map((viewport) => `${route}-${viewport.slug}.png`),
  ),
);

await mkdir(TARGET_DIR, { recursive: true });

let removed = 0;
for (const entry of await readdir(TARGET_DIR)) {
  if (!expected.has(entry)) {
    await rm(path.join(TARGET_DIR, entry));
    removed += 1;
  }
}

for (const name of expected) {
  await copyFile(path.join(SOURCE_DIR, name), path.join(TARGET_DIR, name));
}

console.log(
  `Synced ${expected.size} marketing captures from the Desktop capture set${
    removed > 0 ? `, removed ${removed} stale file(s)` : ""
  }.`,
);
