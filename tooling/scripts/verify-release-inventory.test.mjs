import assert from "node:assert/strict";
import { createHash } from "node:crypto";
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
  const updaterAsset = files.find((name) => name.startsWith("windows-x86_64-"));
  if (files.includes("latest.json") && updaterAsset) {
    await writeFile(path.join(root, "latest.json"), JSON.stringify({
      version: "1.2.3",
      pub_date: "2026-09-21T00:00:00Z",
      platforms: {
        "windows-x86_64": {
          signature: Buffer.from("fixture updater signature").toString("base64"),
          url: `https://github.com/Pytxo-dev/pytxo-releases/releases/download/v1.2.3/${updaterAsset}`,
        },
      },
    }));
  }
  const checksum = files.includes("SHA256SUMS.txt") ? "SHA256SUMS.txt" : "DESKTOP_SHA256SUMS.txt";
  const assets = checksum === "SHA256SUMS.txt"
    ? CLI_FILES.filter((name) => name !== checksum)
    : ["pytxo-desktop-windows-x64.msi"];
  if (files.includes(checksum)) {
    const digest = createHash("sha256").update("release-asset").digest("hex");
    await writeFile(path.join(root, checksum), assets.map((name) => `${digest}  ${name}\n`).join(""));
  }
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

test("rejects a signed Desktop manifest that does not bind its exact updater asset", async () => {
  const files = [...DESKTOP_FILES, "latest.json", "windows-x86_64-installer.msi.zip"];
  const root = await fixture(files);
  const manifestPath = path.join(root, "latest.json");
  const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
  manifest.platforms["windows-x86_64"].url =
    "https://github.com/Pytxo-dev/pytxo-releases/releases/download/v1.2.3/different.msi.zip";
  await writeFile(manifestPath, JSON.stringify(manifest));
  await assert.rejects(
    () => verifyReleaseInventory(root, "desktop", { hasSigningKey: true }),
    /does not bind v1\.2\.3 to windows-x86_64-installer\.msi\.zip/,
  );
});

test("rejects malformed signed Desktop updater metadata", async () => {
  const files = [...DESKTOP_FILES, "latest.json", "windows-x86_64-installer.msi.zip"];
  for (const corruption of ["invalid-json", "invalid-version", "invalid-signature", "extra-platform"]) {
    const root = await fixture(files);
    const manifestPath = path.join(root, "latest.json");
    if (corruption === "invalid-json") {
      await writeFile(manifestPath, "{");
    } else {
      const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
      if (corruption === "invalid-version") manifest.version = "not-semver";
      if (corruption === "invalid-signature") manifest.platforms["windows-x86_64"].signature = "file.sig";
      if (corruption === "extra-platform") manifest.platforms["linux-x86_64"] = manifest.platforms["windows-x86_64"];
      await writeFile(manifestPath, JSON.stringify(manifest));
    }
    await assert.rejects(
      () => verifyReleaseInventory(root, "desktop", { hasSigningKey: true }),
      /Updater manifest/,
      corruption,
    );
  }
});

test("rejects directories and empty assets", async () => {
  const withDirectory = await fixture(CLI_FILES);
  await mkdir(path.join(withDirectory, "nested"));
  await assert.rejects(() => verifyReleaseInventory(withDirectory, "cli"), /files only/);

  const withEmptyAsset = await fixture(CLI_FILES.filter((name) => name !== "SHA256SUMS.txt"));
  await writeFile(path.join(withEmptyAsset, "SHA256SUMS.txt"), "");
  await assert.rejects(() => verifyReleaseInventory(withEmptyAsset, "cli"), /asset is empty/);
});

test("rejects an installer changed after checksums were prepared", async () => {
  const root = await fixture(DESKTOP_FILES);
  await writeFile(path.join(root, "pytxo-desktop-windows-x64.msi"), "different-bytes");
  await assert.rejects(() => verifyReleaseInventory(root, "desktop"), /checksum mismatch/i);
});

test("requires exactly one checksum for each published CLI binary", async () => {
  for (const corruption of ["missing", "duplicate", "unexpected", "malformed"]) {
    const root = await fixture(CLI_FILES);
    const checksumPath = path.join(root, "SHA256SUMS.txt");
    const rows = (await readFile(checksumPath, "utf8")).trimEnd().split("\n");
    if (corruption === "missing") rows.pop();
    if (corruption === "duplicate") rows.push(rows[0]);
    if (corruption === "unexpected") rows.push(rows[0].replace("pytxo-darwin-arm64", "../outside"));
    if (corruption === "malformed") rows[0] = "not-a-checksum";
    await writeFile(checksumPath, rows.join("\n"));
    await assert.rejects(() => verifyReleaseInventory(root, "cli"), /checksum/i, corruption);
  }
});

test("accepts CRLF checksums with the standard binary marker", async () => {
  const root = await fixture(DESKTOP_FILES);
  const checksumPath = path.join(root, "DESKTOP_SHA256SUMS.txt");
  const text = await readFile(checksumPath, "utf8");
  await writeFile(checksumPath, text.replace("  ", " *").replaceAll("\n", "\r\n"));
  assert.deepEqual(await verifyReleaseInventory(root, "desktop"), [...DESKTOP_FILES].sort());
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

test("candidate signing cannot publish a release or update channel", async () => {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const workflow = await readFile(path.resolve(here, "../../.github/workflows/desktop-candidate.yml"), "utf8");
  assert.match(workflow, /on:\s+workflow_dispatch:/);
  assert.match(workflow, /permissions:\s+contents: read/);
  assert.match(workflow, /persist-credentials: false/);
  // Manual only: candidate builds spend billed Windows minutes, so no push trigger.
  assert.doesNotMatch(workflow, /^\s+push:/m);
  assert.match(workflow, /if: github\.repository == 'Pytxo-dev\/pytxo'/);
  assert.doesNotMatch(workflow, /^\s+(pull_request|pull_request_target|schedule|tags):/m);
  assert.doesNotMatch(workflow, /softprops\/action-gh-release|npm publish|git push|gh release/);
  assert.doesNotMatch(workflow, /^\s+(tagName|releaseName|releaseId|releaseBody):/m);
  assert.doesNotMatch(workflow, /PYTXO_RELEASES_TOKEN|NPM_TOKEN/);
  assert.match(workflow, /Missing updater signature/);
  assert.match(workflow, /verify-windows-msi\.ps1/);
  assert.match(workflow, /Get-ChildItem candidate-evidence\/payload -Recurse -File -Filter 'pytxo-desktop\.exe'/);
  assert.match(workflow, /verify-desktop-embedded-assets\.mjs \$payload\.FullName apps\/desktop\/dist/);
  assert.match(workflow, /payload_sha256 = \(Get-FileHash -LiteralPath \$payload\.FullName -Algorithm SHA256\)\.Hash/);
  assert.ok(workflow.indexOf("verify-windows-msi.ps1") < workflow.indexOf("actions/upload-artifact"));
});

test("the private candidate branch does not trigger a Vercel deployment", async () => {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const config = JSON.parse(await readFile(path.resolve(here, "../../apps/web/vercel.json"), "utf8"));
  assert.deepEqual(config.git.deploymentEnabled, { "2ntt/pytxo-beta-candidate-20261003": false });
});

test("the Desktop updater uses a stable signed channel with a release fallback", async () => {
  const here = path.dirname(fileURLToPath(import.meta.url));
  const workflow = await readFile(path.resolve(here, "../../.github/workflows/release.yml"), "utf8");
  const config = JSON.parse(await readFile(
    path.resolve(here, "../../apps/desktop/src-tauri/tauri.conf.json"),
    "utf8",
  ));
  const mirror = jobBlock(workflow, "mirror-public-release");

  assert.deepEqual(config.plugins.updater.endpoints, [
    "https://raw.githubusercontent.com/Pytxo-dev/pytxo-releases/main/latest.json",
    "https://github.com/Pytxo-dev/pytxo-releases/releases/latest/download/latest.json",
  ]);
  assert.match(mirror, /if \[ -f desktop-dist\/latest\.json \]; then/);
  assert.match(mirror, /cp desktop-dist\/latest\.json "\$WORK\/latest\.json"/);
  assert.match(mirror, /preserving the existing updater channel/);
  assert.ok(
    mirror.indexOf("Mirror to public pytxo-releases") <
      mirror.indexOf("Sync install scripts and signed updater channel"),
    "the signed channel must point only to assets already published on the release",
  );
});
