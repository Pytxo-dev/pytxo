import { createHash } from "node:crypto";
import { createReadStream } from "node:fs";
import { readFile, readdir, stat } from "node:fs/promises";
import path from "node:path";
import { pathToFileURL } from "node:url";

const CLI_ASSETS = [
  "SHA256SUMS.txt",
  "pytxo-darwin-arm64",
  "pytxo-darwin-x64",
  "pytxo-linux-arm64",
  "pytxo-linux-x64",
  "pytxo-windows-x64.exe",
];

function sameInventory(actual, expected) {
  return actual.length === expected.length && actual.every((name, index) => name === expected[index]);
}

async function verifyChecksums(directory, checksumName, assets) {
  const checksumPath = path.join(directory, checksumName);
  if ((await stat(checksumPath)).size > 64 * 1024) {
    throw new Error(`Release checksum file is unexpectedly large: ${checksumName}`);
  }
  const expected = new Map();
  const rows = (await readFile(checksumPath, "utf8")).trimEnd().split(/\r?\n/);
  for (const row of rows) {
    const match = row.match(/^([a-fA-F0-9]{64}) [ *](\S+)$/);
    if (!match || !assets.includes(match[2]) || expected.has(match[2])) {
      throw new Error(`Invalid, duplicate, or unexpected release checksum in ${checksumName}`);
    }
    expected.set(match[2], match[1].toLowerCase());
  }
  if (expected.size !== assets.length) {
    throw new Error(`Release checksum coverage is incomplete: ${checksumName}`);
  }
  for (const name of assets) {
    const hash = createHash("sha256");
    for await (const chunk of createReadStream(path.join(directory, name))) hash.update(chunk);
    if (hash.digest("hex") !== expected.get(name)) {
      throw new Error(`Release checksum mismatch: ${name}`);
    }
  }
}

export async function verifyReleaseInventory(directory, kind, { hasSigningKey = false } = {}) {
  const entries = await readdir(directory, { withFileTypes: true });
  const actual = entries.filter((entry) => entry.isFile()).map((entry) => entry.name).sort();
  if (entries.some((entry) => !entry.isFile())) {
    throw new Error(`Release staging must contain files only; found: ${entries.filter((entry) => !entry.isFile()).map((entry) => entry.name).join(", ")}`);
  }

  let expected;
  if (kind === "cli") {
    expected = [...CLI_ASSETS];
  } else if (kind === "desktop") {
    expected = ["DESKTOP_SHA256SUMS.txt", "pytxo-desktop-windows-x64.msi"];
    const updaterAssets = actual.filter((name) => name.startsWith("windows-x86_64-"));
    if (hasSigningKey) {
      if (updaterAssets.length !== 1) {
        throw new Error(`Signed Desktop publication requires exactly one Windows updater asset; found ${updaterAssets.length}`);
      }
      expected.push("latest.json", updaterAssets[0]);
    }
  } else {
    throw new Error(`Unknown release inventory kind: ${kind}`);
  }

  expected.sort();
  if (!sameInventory(actual, expected)) {
    throw new Error(`Unexpected ${kind} release inventory\nexpected: ${expected.join(", ")}\nactual:   ${actual.join(", ")}`);
  }

  for (const name of expected) {
    const metadata = await stat(path.join(directory, name));
    if (metadata.size === 0) throw new Error(`Release asset is empty: ${name}`);
  }
  // Match the existing published checksum contract: five CLI binaries, or the
  // manual Desktop MSI. Tauri verifies updater signatures separately.
  await verifyChecksums(
    directory,
    kind === "cli" ? "SHA256SUMS.txt" : "DESKTOP_SHA256SUMS.txt",
    kind === "cli" ? CLI_ASSETS.filter((name) => name !== "SHA256SUMS.txt") : ["pytxo-desktop-windows-x64.msi"],
  );
  return actual;
}

const invoked = process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href;
if (invoked) {
  const [kind, directory] = process.argv.slice(2);
  if (!kind || !directory) throw new Error("Usage: verify-release-inventory.mjs <cli|desktop> <directory>");
  const files = await verifyReleaseInventory(path.resolve(directory), kind, {
    hasSigningKey: process.env.HAS_SIGNING_KEY === "true",
  });
  console.log(`Verified exact ${kind} release inventory: ${files.join(", ")}`);
}
