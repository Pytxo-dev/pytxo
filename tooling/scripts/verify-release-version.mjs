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

async function capturedVersion(file, pattern, label, { optional = false } = {}) {
  const content = await readFile(file, "utf8");
  const value = content.match(pattern)?.[1];
  if (!value && !optional) throw new Error(`Missing ${label} version in ${file}`);
  return value;
}

export async function verifyReleaseVersion(root, expected) {
  const cargo = await readFile(path.join(root, "Cargo.toml"), "utf8");
  const rust = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (!rust) throw new Error("Missing workspace package version in Cargo.toml");
  const siteFile = path.join(root, "apps", "web", "src", "lib", "site.ts");
  const site = await readFile(siteFile, "utf8");
  const published = await capturedVersion(siteFile, /^export const PUBLISHED_VERSION = "([^"]+)";/m, "published website");
  if (!/^export const PYTXO_VERSION = PUBLISHED_VERSION;/m.test(site)) {
    throw new Error("Website download alias must use PUBLISHED_VERSION");
  }

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
      siteFile,
      /^export const CANDIDATE_VERSION = "([^"]+)";/m,
      "web source candidate",
    ),
    published,
    installer: await capturedVersion(
      path.join(root, "distribution", "pytxo-releases", "install.ps1"),
      /\(v([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?) provides Windows x64\)/,
      "PowerShell installer",
    ),
    // `npx pytxo` needs no version; a pinned install line must be the published one.
    readme: await capturedVersion(
      path.join(root, "README.md"),
      /npm i -g pytxo@([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)/,
      "README install",
      { optional: true },
    ),
    action: await capturedVersion(
      path.join(root, "actions", "land", "action.yml"),
      /default: "([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)" # pytxo-version/,
      "GitHub Action CLI pin",
    ),
    claudePlugin: await jsonVersion(path.join(root, "integrations", "claude-code", ".claude-plugin", "plugin.json")),
    claudeHook: await capturedVersion(
      path.join(root, "integrations", "claude-code", "hooks", "hooks.json"),
      /npx -y pytxo@([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?) hook/,
      "Claude Code hook pin",
    ),
    codexHook: await capturedVersion(
      path.join(root, "integrations", "codex", "hooks.json"),
      /npx -y pytxo@([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?) hook/,
      "Codex hook pin",
    ),
    changelog: await capturedVersion(
      path.join(root, "apps", "web", "content", "docs", "reference", "changelog.mdx"),
      /This source candidate targets \*\*([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)\*\*/,
      "changelog source candidate",
    ),
    changelogPublished: await capturedVersion(
      path.join(root, "apps", "web", "content", "docs", "reference", "changelog.mdx"),
      /The latest public (?:GitHub and npm )?release is \**([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)/,
      "changelog latest public release",
    ),
    releaseNotes: await capturedVersion(
      path.join(root, "distribution", "release-notes", `v${expected ?? rust}.md`),
      /^# Pytxo v([0-9]+\.[0-9]+\.[0-9]+(?:[-+][0-9A-Za-z.-]+)?)/m,
      "release notes",
    ),
  };
  for (const [name, version] of Object.entries(versions)) {
    if (version === undefined) delete versions[name];
  }
  expected ??= versions.rust;
  // The checked-in installer/release notes target the next candidate. Public
  // install instructions must remain on the published tag until promotion.
  const publishedSurfaces = new Set(["published", "readme", "changelogPublished"]);
  const mismatches = Object.entries(versions)
    .filter(([name, version]) => !publishedSurfaces.has(name) && version !== expected)
    .map(([name, version]) => `${name}=${version}`);
  if (mismatches.length) {
    throw new Error(`Release version must be ${expected}; found ${mismatches.join(", ")}`);
  }
  const publicMismatches = Object.entries(versions)
    .filter(([name, version]) => publishedSurfaces.has(name) && version !== published)
    .map(([name, version]) => `${name}=${version}`);
  if (publicMismatches.length) {
    throw new Error(`Published version must be ${published}; found ${publicMismatches.join(", ")}`);
  }
  return versions;
}

const invoked = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (invoked) {
  const expected = process.argv[2];
  const root = path.resolve(import.meta.dirname, "..", "..");
  const versions = await verifyReleaseVersion(root, expected);
  console.log(`Verified candidate Pytxo ${versions.rust} and declared published ${versions.published}: ${Object.keys(versions).join(", ")}. Public availability was not checked.`);
}
