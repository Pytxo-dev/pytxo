import assert from "node:assert/strict";
import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

import { verifyReleaseVersion } from "./verify-release-version.mjs";

async function fixture(versions) {
  const root = await mkdtemp(path.join(tmpdir(), "pytxo-version-"));
  await mkdir(path.join(root, "apps", "desktop", "src-tauri"), { recursive: true });
  await mkdir(path.join(root, "apps", "demo-video"), { recursive: true });
  await mkdir(path.join(root, "apps", "web", "src", "lib"), { recursive: true });
  await mkdir(path.join(root, "apps", "web", "content", "docs", "reference"), { recursive: true });
  await mkdir(path.join(root, "packages", "pytxo"), { recursive: true });
  await mkdir(path.join(root, "distribution", "pytxo-releases"), { recursive: true });
  await mkdir(path.join(root, "distribution", "release-notes"), { recursive: true });
  await writeFile(path.join(root, "Cargo.toml"), `[workspace.package]\nversion = "${versions.rust}"\n`);
  const rustPackages = ["pytxo-catalog", "pytxo-cli", "pytxo-core", "pytxo-desktop", "pytxo-mcp", "pytxo-orchestrate", "pytxo-planner", "pytxo-runner", "pytxo-sanitize", "pytxo-scheduler", "pytxo-shell", "pytxo-signal", "pytxo-store", "pytxo-tui", "pytxo-voice"];
  await writeFile(
    path.join(root, "Cargo.lock"),
    rustPackages.map((name) => `[[package]]\nname = "${name}"\nversion = "${name === "pytxo-cli" ? versions.cargoLock ?? versions.rust : versions.rust}"\n`).join("\n"),
  );
  await writeFile(path.join(root, "apps", "desktop", "package.json"), JSON.stringify({ version: versions.desktop }));
  await writeFile(path.join(root, "apps", "desktop", "package-lock.json"), JSON.stringify({ version: versions.desktopLock ?? versions.desktop, packages: { "": { version: versions.desktopLock ?? versions.desktop } } }));
  await writeFile(path.join(root, "apps", "desktop", "src-tauri", "tauri.conf.json"), JSON.stringify({ version: versions.tauri }));
  await writeFile(path.join(root, "apps", "demo-video", "package.json"), JSON.stringify({ version: versions.demo }));
  await writeFile(path.join(root, "apps", "demo-video", "package-lock.json"), JSON.stringify({ version: versions.demoLock ?? versions.demo, packages: { "": { version: versions.demoLock ?? versions.demo } } }));
  await writeFile(path.join(root, "packages", "pytxo", "package.json"), JSON.stringify({ version: versions.npm }));
  await writeFile(path.join(root, "apps", "web", "src", "lib", "site.ts"), `export const PUBLISHED_VERSION = "${versions.publicRelease ?? versions.rust}";\nexport const CANDIDATE_VERSION = "${versions.web ?? versions.rust}";\nexport const PYTXO_VERSION = PUBLISHED_VERSION;\n`);
  await writeFile(path.join(root, "distribution", "pytxo-releases", "install.ps1"), `throw "(v${versions.installer ?? versions.rust} provides Windows x64)"\n`);
  await writeFile(path.join(root, "README.md"), `npm i -g pytxo@${versions.readme ?? versions.publicRelease ?? versions.rust}\n`);
  await writeFile(path.join(root, "apps", "web", "content", "docs", "reference", "changelog.mdx"), `This source candidate targets **${versions.changelog ?? versions.rust}**.\nThe latest public release is **${versions.publicRelease ?? versions.rust}**.\n`);
  await writeFile(path.join(root, "distribution", "release-notes", `v${versions.rust}.md`), `# Pytxo v${versions.releaseNotes ?? versions.rust}\n`);
  return root;
}

test("accepts one shared version across every release-facing surface", async () => {
  const root = await fixture({ rust: "1.1.1", desktop: "1.1.1", tauri: "1.1.1", demo: "1.1.1", npm: "1.1.1" });
  const actual = await verifyReleaseVersion(root, "1.1.1");
  assert.ok(Object.values(actual).every((version) => version === "1.1.1"));
});

test("rejects stale lockfiles and public download copy", async () => {
  const root = await fixture({
    rust: "1.1.1",
    cargoLock: "1.1.0",
    desktop: "1.1.1",
    desktopLock: "1.1.0",
    tauri: "1.1.1",
    demo: "1.1.1",
    npm: "1.1.1",
    web: "1.1.0",
  });
  await assert.rejects(
    () => verifyReleaseVersion(root, "1.1.1"),
    /cargoLock:pytxo-cli=1\.1\.0, desktopLock=1\.1\.0, web=1\.1\.0/,
  );
});

test("rejects disagreeing package-lock top-level and root package versions", async () => {
  const root = await fixture({ rust: "1.1.1", desktop: "1.1.1", tauri: "1.1.1", demo: "1.1.1", npm: "1.1.1" });
  await writeFile(
    path.join(root, "apps", "desktop", "package-lock.json"),
    JSON.stringify({ version: "1.1.0", packages: { "": { version: "1.1.1" } } }),
  );
  await assert.rejects(() => verifyReleaseVersion(root, "1.1.1"), /Package-lock versions disagree/);
});

test("rejects a checked-in npm wrapper that differs from the release", async () => {
  const root = await fixture({ rust: "1.1.1", desktop: "1.1.1", tauri: "1.1.1", demo: "1.1.1", npm: "1.1.0" });
  await assert.rejects(() => verifyReleaseVersion(root, "1.1.1"), /npm=1\.1\.0/);
});

test("accepts the prerelease grammar allowed by the release workflow", async () => {
  const version = "1.2.2-rc.1";
  const root = await fixture({ rust: version, desktop: version, tauri: version, demo: version, npm: version });
  const actual = await verifyReleaseVersion(root, version);
  assert.ok(Object.values(actual).every((value) => value === version));
});

test("validates the candidate independently of the older published release", async () => {
  const root = await fixture({ rust: "1.2.2", desktop: "1.2.2", tauri: "1.2.2", demo: "1.2.2", npm: "1.2.2", publicRelease: "1.2.1" });
  const actual = await verifyReleaseVersion(root);
  assert.equal(actual.changelog, "1.2.2");
  assert.equal(actual.web, "1.2.2");
  assert.equal(actual.published, "1.2.1");
  assert.equal(actual.readme, "1.2.1");
});

test("rejects README install instructions that advertise an unpublished candidate", async () => {
  const root = await fixture({ rust: "1.2.2", desktop: "1.2.2", tauri: "1.2.2", demo: "1.2.2", npm: "1.2.2", publicRelease: "1.2.1", readme: "1.2.2" });
  await assert.rejects(() => verifyReleaseVersion(root), /Published version must be 1\.2\.1; found readme=1\.2\.2/);
});

test("rejects a download alias that points at the candidate", async () => {
  const root = await fixture({ rust: "1.2.2", desktop: "1.2.2", tauri: "1.2.2", demo: "1.2.2", npm: "1.2.2" });
  await writeFile(path.join(root, "apps", "web", "src", "lib", "site.ts"), 'export const PUBLISHED_VERSION = "1.2.1";\nexport const CANDIDATE_VERSION = "1.2.2";\nexport const PYTXO_VERSION = CANDIDATE_VERSION;\n');
  await assert.rejects(() => verifyReleaseVersion(root), /download alias must use PUBLISHED_VERSION/);
});

test("does not substitute the published tag for a missing website candidate", async () => {
  const root = await fixture({ rust: "1.2.2", desktop: "1.2.2", tauri: "1.2.2", demo: "1.2.2", npm: "1.2.2" });
  await writeFile(path.join(root, "apps", "web", "src", "lib", "site.ts"), 'export const PUBLISHED_VERSION = "1.2.2";\nexport const PYTXO_VERSION = PUBLISHED_VERSION;\n');
  await assert.rejects(() => verifyReleaseVersion(root), /Missing web source candidate version/);
});

test("rejects a stale candidate even when the published version matches", async () => {
  const root = await fixture({ rust: "1.2.2", desktop: "1.2.2", tauri: "1.2.2", demo: "1.2.2", npm: "1.2.2", changelog: "1.2.1", publicRelease: "1.2.2" });
  await assert.rejects(() => verifyReleaseVersion(root), /changelog=1\.2\.1/);
});

test("does not substitute a public version for a missing candidate declaration", async () => {
  const root = await fixture({ rust: "1.2.2", desktop: "1.2.2", tauri: "1.2.2", demo: "1.2.2", npm: "1.2.2" });
  await writeFile(path.join(root, "apps", "web", "content", "docs", "reference", "changelog.mdx"), "Current public binary: **1.2.2**.\n");
  await assert.rejects(() => verifyReleaseVersion(root), /Missing changelog source candidate version/);
});
