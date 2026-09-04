import assert from "node:assert/strict";
import { mkdtemp, mkdir, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";

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
