import path from "node:path";
import process from "node:process";
import {createHash} from "node:crypto";
import {readFile} from "node:fs/promises";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const manifestPath = path.join(appRoot, "r3-storyboard-props.json");
const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const errors = [];
const expectedKeys = ["running", "completed", "review", "applied"];
const shaPattern = /^[a-f0-9]{64}$/;

const probeDimensions = (assetPath) => {
  const result = spawnSync("ffprobe", [
    "-v",
    "error",
    "-select_streams",
    "v:0",
    "-show_entries",
    "stream=width,height",
    "-of",
    "json",
    assetPath,
  ], {
    cwd: appRoot,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    throw new Error(`ffprobe failed for ${assetPath}:\n${result.stderr || result.stdout}`);
  }
  const stream = JSON.parse(result.stdout).streams?.[0];
  return {width: stream?.width, height: stream?.height};
};

if (manifest.schemaVersion !== 1) errors.push("schemaVersion must be 1");
if (manifest.kind !== "r3-native-still-storyboard") errors.push("kind is invalid");
for (const field of ["msiSha256", "executableSha256", "packageDigest"]) {
  if (!shaPattern.test(manifest.candidate?.[field] ?? "")) {
    errors.push(`candidate.${field} must be a lowercase SHA-256`);
  }
}
if (manifest.evidenceBoundary?.continuousFootage !== false) {
  errors.push("continuousFootage must remain false until continuous R3 capture exists");
}
if (manifest.evidenceBoundary?.pointerMotion !== false) {
  errors.push("pointerMotion must remain false for still-source footage");
}
if (manifest.evidenceBoundary?.native4kSource !== false) {
  errors.push("native4kSource must remain false for the 1282x802 inputs");
}

const assets = Array.isArray(manifest.assets) ? manifest.assets : [];
const keys = assets.map((asset) => asset.key);
if (JSON.stringify([...keys].sort()) !== JSON.stringify([...expectedKeys].sort())) {
  errors.push(`asset keys must be exactly: ${expectedKeys.join(", ")}`);
}

for (const asset of assets) {
  if (!asset.path?.startsWith("product/r3-storyboard/") || path.isAbsolute(asset.path)) {
    errors.push(`${asset.key}: asset path must stay under product/r3-storyboard`);
    continue;
  }
  const absolute = path.join(appRoot, "public", asset.path);
  const relative = path.relative(path.join(appRoot, "public", "product", "r3-storyboard"), absolute);
  if (relative.startsWith("..") || path.isAbsolute(relative)) {
    errors.push(`${asset.key}: asset path escapes the storyboard directory`);
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
  for (const mask of asset.privacyMasks ?? []) {
    const values = [mask.x, mask.y, mask.width, mask.height];
    if (!values.every((value) => Number.isFinite(value) && value >= 0)) {
      errors.push(`${asset.key}: privacy mask has invalid coordinates`);
      continue;
    }
    if (mask.width <= 0 || mask.height <= 0 || mask.x + mask.width > asset.width || mask.y + mask.height > asset.height) {
      errors.push(`${asset.key}: privacy mask is outside the source image`);
    }
    if (!mask.label) errors.push(`${asset.key}: privacy mask needs a disclosure label`);
  }
}

for (const key of ["review", "applied"]) {
  const asset = assets.find((candidate) => candidate.key === key);
  if (!asset || (asset.privacyMasks?.length ?? 0) === 0) {
    errors.push(`${key}: disposable host path must have a privacy mask`);
  }
}

if (errors.length > 0) {
  process.stderr.write(`R3 storyboard validation failed:\n${errors.map((error) => `- ${error}`).join("\n")}\n`);
  process.exit(1);
}

process.stdout.write(
  `R3 storyboard validation passed: ${assets.length} exact native stills, ${assets.reduce((sum, asset) => sum + (asset.privacyMasks?.length ?? 0), 0)} bounded privacy masks, continuous-footage claim disabled.\n`,
);
