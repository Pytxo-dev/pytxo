import assert from "node:assert/strict";
import { mkdtemp, mkdir, readFile, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { verifyReleaseInventory } from "./verify-release-inventory.mjs";

const CLI_FILES = [
  "SHA256SUMS.txt",
  "pytxo-darwin-arm64",
  "pytxo-darwin-x64",
  "pytxo-linux-arm64",
  "pytxo-linux-x64",
  "pytxo-windows-x64.exe",
];
const DESKTOP_FILES = ["DESKTOP_SHA256SUMS.txt", "pytxo-desktop-windows-x64.msi"];

async function fixture(files) {
  const root = await mkdtemp(path.join(tmpdir(), "pytxo-release-inventory-"));
  await Promise.all(files.map((name) => writeFile(path.join(root, name), "release-asset")));
  return root;
}

function jobBlock(workflow, name) {
  const marker = `  ${name}:`;
  const start = workflow.indexOf(marker);
  assert.notEqual(start, -1, `missing workflow job: ${name}`);
  const remainder = workflow.slice(start + marker.length);
  const nextJob = remainder.search(/^  [a-z0-9-]+:\s*$/m);
  return nextJob === -1 ? workflow.slice(start) : workflow.slice(start, start + marker.length + nextJob);
}

function assertNeeds(job, expected) {
  const match = job.match(/^    needs: \[([^\]]+)\]$/m);
  assert.ok(match, "job must declare an explicit needs list");
  const actual = match[1].split(",").map((entry) => entry.trim()).sort();
  assert.deepEqual(actual, [...expected].sort());
}

test("accepts the exact five-platform CLI inventory and checksum", async () => {
  const root = await fixture(CLI_FILES);
  assert.deepEqual(await verifyReleaseInventory(root, "cli"), [...CLI_FILES].sort());
});

test("rejects a stale CLI asset in CLI staging", async () => {
  const root = await fixture([...CLI_FILES, "pytxo-old-platform"]);
  await assert.rejects(() => verifyReleaseInventory(root, "cli"), /Unexpected cli release inventory/);
});

test("accepts unsigned installer-only Desktop staging", async () => {
  const root = await fixture(DESKTOP_FILES);
  assert.deepEqual(await verifyReleaseInventory(root, "desktop"), [...DESKTOP_FILES].sort());
});

test("accepts a signed Desktop updater inventory", async () => {
  const files = [...DESKTOP_FILES, "latest.json", "windows-x86_64-installer.msi.zip"];
  const root = await fixture(files);
  assert.deepEqual(
    await verifyReleaseInventory(root, "desktop", { hasSigningKey: true }),
    [...files].sort(),
  );
});

test("rejects stale CLI and checksum files in Desktop staging", async () => {
  const root = await fixture([...DESKTOP_FILES, "pytxo-windows-x64.exe", "SHA256SUMS.txt"]);
  await assert.rejects(() => verifyReleaseInventory(root, "desktop"), /Unexpected desktop release inventory/);
});

test("rejects signed Desktop staging without updater evidence", async () => {
  const root = await fixture([...DESKTOP_FILES, "latest.json"]);
  await assert.rejects(
    () => verifyReleaseInventory(root, "desktop", { hasSigningKey: true }),
    /requires exactly one Windows updater asset/,
  );
});

test("rejects directories and empty assets", async () => {
  const withDirectory = await fixture(CLI_FILES);
  await mkdir(path.join(withDirectory, "nested"));
  await assert.rejects(() => verifyReleaseInventory(withDirectory, "cli"), /files only/);

  const withEmptyAsset = await fixture(CLI_FILES.filter((name) => name !== "SHA256SUMS.txt"));
  await writeFile(path.join(withEmptyAsset, "SHA256SUMS.txt"), "");
  await assert.rejects(() => verifyReleaseInventory(withEmptyAsset, "cli"), /asset is empty/);
});

test("release publication waits for exact CLI and Desktop preparation", async () => {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const workflow = await readFile(path.resolve(here, "../../.github/workflows/release.yml"), "utf8");
  const cli = jobBlock(workflow, "prepare-cli-release");
  const desktopBuild = jobBlock(workflow, "desktop-release");
  const desktop = jobBlock(workflow, "prepare-desktop-release");
  const privateRelease = jobBlock(workflow, "publish-private-release");
  const publicRelease = jobBlock(workflow, "mirror-public-release");
  const npm = jobBlock(workflow, "npm");

  assert.match(cli, /Prepare clean release staging/);
  assert.match(cli, /verify-release-inventory\.mjs cli dist/);
  assert.doesNotMatch(cli, /softprops\/action-gh-release/);

  assert.match(desktopBuild, /^    needs: prepare-cli-release$/m);
  assert.doesNotMatch(desktopBuild, /^\s+tagName:|^\s+releaseName:|^\s+releaseId:/m);
  assert.doesNotMatch(desktopBuild, /softprops\/action-gh-release/);
  assert.match(desktop, /Prepare clean Desktop staging/);
  assert.match(desktop, /verify-release-inventory\.mjs desktop desktop-dist/);
  assert.ok(
    desktop.indexOf("verify-release-inventory.mjs desktop desktop-dist") <
      desktop.indexOf("Upload exact Desktop release assets"),
    "Desktop inventory must be verified before its prepared artifact is exposed downstream",
  );

  assertNeeds(privateRelease, ["prepare-cli-release", "prepare-desktop-release"]);
  assertNeeds(publicRelease, [
    "prepare-cli-release",
    "prepare-desktop-release",
    "publish-private-release",
  ]);
  assertNeeds(npm, [
    "prepare-cli-release",
    "prepare-desktop-release",
    "publish-private-release",
    "mirror-public-release",
  ]);
  for (const publication of [privateRelease, publicRelease]) {
    assert.match(publication, /verify-release-inventory\.mjs cli cli-dist/);
    assert.match(publication, /verify-release-inventory\.mjs desktop desktop-dist/);
    assert.ok(
      publication.indexOf("verify-release-inventory.mjs desktop desktop-dist") <
        publication.indexOf("softprops/action-gh-release"),
      "both inventories must be verified before a GitHub release action",
    );
  }
  assert.equal(workflow.match(/softprops\/action-gh-release/g)?.length, 2);
  assert.equal(workflow.match(/npm publish --access public/g)?.length, 1);
});
