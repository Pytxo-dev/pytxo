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
  await mkdir(path.join(root, "packages", "pytxo"), { recursive: true });
  await writeFile(path.join(root, "Cargo.toml"), `[workspace.package]\nversion = "${versions.rust}"\n`);
  await writeFile(path.join(root, "apps", "desktop", "package.json"), JSON.stringify({ version: versions.desktop }));
  await writeFile(path.join(root, "apps", "desktop", "src-tauri", "tauri.conf.json"), JSON.stringify({ version: versions.tauri }));
  await writeFile(path.join(root, "apps", "demo-video", "package.json"), JSON.stringify({ version: versions.demo }));
  await writeFile(path.join(root, "packages", "pytxo", "package.json"), JSON.stringify({ version: versions.npm }));
  return root;
}

test("accepts one shared Rust, Desktop, Tauri, demo, and npm version", async () => {
  const root = await fixture({ rust: "1.1.1", desktop: "1.1.1", tauri: "1.1.1", demo: "1.1.1", npm: "1.1.1" });
  assert.deepEqual(await verifyReleaseVersion(root, "1.1.1"), {
    rust: "1.1.1",
    desktop: "1.1.1",
    tauri: "1.1.1",
    demo: "1.1.1",
    npm: "1.1.1",
  });
});

test("rejects a checked-in npm wrapper that differs from the release", async () => {
  const root = await fixture({ rust: "1.1.1", desktop: "1.1.1", tauri: "1.1.1", demo: "1.1.1", npm: "1.1.0" });
  await assert.rejects(() => verifyReleaseVersion(root, "1.1.1"), /npm=1\.1\.0/);
});
