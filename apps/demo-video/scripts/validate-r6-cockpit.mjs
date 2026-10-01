import path from "node:path";
import process from "node:process";
import {createHash} from "node:crypto";
import {readFile} from "node:fs/promises";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const assetRoot = path.join(appRoot, "public", "product", "r6-cockpit");
const manifest = JSON.parse(
  await readFile(path.join(appRoot, "r6-cockpit-props.json"), "utf8"),
);
const errors = [];
const shaPattern = /^[a-f0-9]{64}$/;
const expectedKeys = ["first-use", "dense-canvas"];

const probeDimensions = (assetPath) => {
  const result = spawnSync(
    "ffprobe",
    [
      "-v",
      "error",
      "-select_streams",
      "v:0",
      "-show_entries",
      "stream=width,height",
      "-of",
      "json",
      assetPath,
    ],
    {cwd: appRoot, encoding: "utf8", maxBuffer: 8 * 1024 * 1024},
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`ffprobe failed for ${assetPath}:\n${result.stderr || result.stdout}`);
  }
  const stream = JSON.parse(result.stdout).streams?.[0];
  return {width: stream?.width, height: stream?.height};
};

if (manifest.schemaVersion !== 1) errors.push("schemaVersion must be 1");
if (manifest.kind !== "r6-native-still-cockpit-proof") errors.push("kind is invalid");
for (const field of [
  "msiSha256",
  "executableSha256",
  "candidateReceiptSha256",
  "stateReceiptSha256",
]) {
  if (!shaPattern.test(manifest.candidate?.[field] ?? "")) {
    errors.push(`candidate.${field} must be a lowercase SHA-256`);
  }
}

const boundary = manifest.evidenceBoundary ?? {};
if (boundary.continuousFootage !== false) {
  errors.push("continuousFootage must remain false until uninterrupted R6 capture exists");
}
if (boundary.pointerMotion !== false) {
  errors.push("pointerMotion must remain false for still-source footage");
}
if (boundary.native4kSource !== false) {
  errors.push("native4kSource must remain false for 1602x1002 source frames");
}
if (boundary.windowsScalePercent !== 125 || boundary.windowDpi !== 120) {
  errors.push("R6 source-scale evidence must remain bound to 125% / 120 DPI");
}
if (!boundary.label?.includes("exact native stills")) {
  errors.push("evidenceBoundary.label must disclose the exact native still source");
}

const observed = manifest.observed ?? {};
if (observed.workers !== 24 || observed.waves !== 8 || observed.fitPercent !== 39) {
  errors.push("dense-canvas observations must remain 24 workers, 8 waves and 39% Fit");
}
if (observed.minimapVisible !== true || observed.redundantCommitRailAbsent !== true) {
  errors.push("dense-canvas minimap and redundant-rail observations are incomplete");
}

const assets = Array.isArray(manifest.assets) ? manifest.assets : [];
const keys = assets.map((asset) => asset.key);
if (JSON.stringify([...keys].sort()) !== JSON.stringify([...expectedKeys].sort())) {
  errors.push(`asset keys must be exactly: ${expectedKeys.join(", ")}`);
}

for (const asset of assets) {
  if (!asset.path?.startsWith("product/r6-cockpit/") || path.isAbsolute(asset.path)) {
    errors.push(`${asset.key}: asset path must stay under product/r6-cockpit`);
    continue;
  }
  const absolute = path.join(appRoot, "public", asset.path);
  const relative = path.relative(assetRoot, absolute);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    errors.push(`${asset.key}: asset path escapes the R6 cockpit directory`);
    continue;
  }
  let bytes;
  try {
    bytes = await readFile(absolute);
  } catch (error) {
    errors.push(`${asset.key}: missing asset (${error.message})`);
    continue;
  }
  const sha256 = createHash("sha256").update(bytes).digest("hex");
  if (sha256 !== asset.sha256) errors.push(`${asset.key}: SHA-256 mismatch`);
  const dimensions = probeDimensions(absolute);
  if (dimensions.width !== asset.width || dimensions.height !== asset.height) {
    errors.push(
      `${asset.key}: expected ${asset.width}x${asset.height}, received ${dimensions.width}x${dimensions.height}`,
    );
  }
}

if (errors.length > 0) {
  process.stderr.write(
    `R6 cockpit proof validation failed:\n${errors.map((error) => `- ${error}`).join("\n")}\n`,
  );
  process.exit(1);
}

process.stdout.write(
  "R6 cockpit proof validation passed: 2 exact native stills, 24 recorded workers, 8 waves, continuous-footage claim disabled.\n",
);
