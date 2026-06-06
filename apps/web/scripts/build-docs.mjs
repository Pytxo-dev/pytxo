import { execSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const monorepoDocs = path.join(root, "../docs");
const bundleDir = path.join(root, ".docs-bundle");

function resolveDocsDir() {
  if (fs.existsSync(path.join(monorepoDocs, "package.json"))) {
    return monorepoDocs;
  }
  if (fs.existsSync(path.join(bundleDir, "package.json"))) {
    return bundleDir;
  }
  throw new Error(
    "Docusaurus source not found. Run: node scripts/bundle-docs.mjs (before CLI deploy)",
  );
}

const docsDir = resolveDocsDir();
console.log(`build-docs: building in ${docsDir}`);
execSync("npm ci && npm run build", {
  cwd: docsDir,
  stdio: "inherit",
  env: { ...process.env, DOCS_BUILD_DIR: path.join(docsDir, "build") },
});
