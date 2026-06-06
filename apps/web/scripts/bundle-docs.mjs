import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const monorepoDocs = path.join(root, "../docs");
const bundleDir = path.join(root, ".docs-bundle");

const SKIP = new Set(["node_modules", "build", ".docusaurus"]);

function copyDocsSource(src, dest) {
  fs.rmSync(dest, { recursive: true, force: true });
  fs.mkdirSync(dest, { recursive: true });

  for (const entry of fs.readdirSync(src, { withFileTypes: true })) {
    if (SKIP.has(entry.name)) continue;
    const from = path.join(src, entry.name);
    const to = path.join(dest, entry.name);
    if (entry.isDirectory()) {
      fs.cpSync(from, to, { recursive: true });
    } else {
      fs.copyFileSync(from, to);
    }
  }
}

if (!fs.existsSync(path.join(monorepoDocs, "package.json"))) {
  if (!fs.existsSync(path.join(bundleDir, "package.json"))) {
    throw new Error(
      "Docusaurus source not found. Expected apps/docs or apps/web/.docs-bundle",
    );
  }
  console.log("bundle-docs: using existing .docs-bundle");
} else {
  copyDocsSource(monorepoDocs, bundleDir);
  console.log(`bundle-docs: ${monorepoDocs} → ${bundleDir}`);
}
