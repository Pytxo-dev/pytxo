import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");

const candidates = [
  process.env.DOCS_BUILD_DIR,
  path.join(root, "../docs/build"),
  path.join(root, ".docs-bundle/build"),
].filter(Boolean);

const docsBuild = candidates.find((dir) => fs.existsSync(dir));
const target = path.join(root, "public/docs");

if (!docsBuild) {
  throw new Error(
    `Docs build not found. Tried: ${candidates.join(", ")}. Run npm run build:docs first.`,
  );
}

fs.rmSync(target, { recursive: true, force: true });
fs.cpSync(docsBuild, target, { recursive: true });
console.log(`copy-docs: ${docsBuild} → ${target}`);
