import { readFile } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

async function jsonVersion(file) {
  const value = JSON.parse(await readFile(file, "utf8")).version;
  if (typeof value !== "string" || !value) throw new Error(`Missing version in ${file}`);
  return value;
}

async function packageLockVersion(file) {
  const lock = JSON.parse(await readFile(file, "utf8"));
  const rootVersion = lock.packages?.[""]?.version;
  const topLevelVersion = lock.version;
  if (typeof rootVersion !== "string" || !rootVersion || typeof topLevelVersion !== "string" || !topLevelVersion) {
    throw new Error(`Missing package versions in ${file}`);
  }
  if (rootVersion !== topLevelVersion) {
    throw new Error(`Package-lock versions disagree in ${file}: root=${rootVersion}, top-level=${topLevelVersion}`);
  }
  return rootVersion;
}

const VERSIONED_RUST_PACKAGES = [
  "pytxo-catalog",
  "pytxo-cli",
  "pytxo-core",
  "pytxo-desktop",
  "pytxo-mcp",
  "pytxo-orchestrate",
  "pytxo-planner",
  "pytxo-runner",
  "pytxo-sanitize",
  "pytxo-scheduler",
  "pytxo-shell",
  "pytxo-signal",
  "pytxo-store",
  "pytxo-tui",
  "pytxo-voice",
];

async function cargoLockVersions(file) {
  const content = await readFile(file, "utf8");
  return Object.fromEntries(VERSIONED_RUST_PACKAGES.map((name) => {
    const escaped = name.replaceAll("-", "\\-");
    const version = content.match(new RegExp(`name = "${escaped}"\\r?\\nversion = "([^"]+)"`))?.[1];
    if (!version) throw new Error(`Missing ${name} version in ${file}`);
    return [`cargoLock:${name}`, version];
  }));
}

async function capturedVersion(file, pattern, label) {
  const content = await readFile(file, "utf8");
  const value = content.match(pattern)?.[1];
  if (!value) throw new Error(`Missing ${label} version in ${file}`);
  return value;
}

export async function verifyReleaseVersion(root, expected) {
  const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
  const rust = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!rust) throw new Error("Missing workspace package version in Cargo.toml");

  const versions = {
    rust,
    ...await cargoLockVersions(path.join(root, "Cargo.lock")),
    desktop: await jsonVersion(path.join(root, "apps", "desktop", "package.json")),
    desktopLock: await packageLockVersion(path.join(root, "apps", "desktop", "package-lock.json")),
    tauri: await jsonVersion(path.join(root, "apps", "desktop", "src-tauri", "tauri.conf.json")),
    demo: await jsonVersion(path.join(root, "apps", "demo-video", "package.json")),
    demoLock: await packageLockVersion(path.join(root, "apps", "demo-video", "package-lock.json")),
    npm: await jsonVersion(path.join(root, "packages", "pytxo", "package.json")),
    web: await capturedVersion(
      path.join(root, "apps", "web", "src", "lib", "site.ts"),
      /PYTXO_VERSION = "([^"]+)"/,
      "web download",
    ),
    installer: await capturedVersion(
      path.join(root, "distribution", "pytxo-releases", "install.ps1"),
      /\(v([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?) provides Windows x64\)/,
      "PowerShell installer",
    ),
    readme: await capturedVersion(
      path.join(root, "README.md"),
      /npm i -g pytxo@([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)/,
      "README install",
    ),
    changelog: await capturedVersion(
      path.join(root, "apps", "web", "content", "docs", "reference", "changelog.mdx"),
      /Current public binary: \*\*([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)\*\*/,
      "public changelog",
    ),
    releaseNotes: await capturedVersion(
      path.join(root, "distribution", "release-notes", `v${expected ?? rust}.md`),
      /^# Pytxo v([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)/m,
      "release notes",
    ),
  };
  expected ??= versions.rust;
  const mismatches = Object.entries(versions)
    .filter(([, version]) => version !== expected)
    .map(([name, version]) => `${name}=${version}`);
  if (mismatches.length) {
    throw new Error(`Release version must be ${expected}; found ${mismatches.join(", ")}`);
  }
  return versions;
}

const invoked = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (invoked) {
  const expected = process.argv[2];
  const root = path.resolve(import.meta.dirname, "..", "..");
  const versions = await verifyReleaseVersion(root, expected);
  console.log(`Verified Pytxo ${versions.rust}: ${Object.keys(versions).join(", ")}`);
}
