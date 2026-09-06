import {readFile, stat} from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {spawnSync} from "node:child_process";
import {fileURLToPath} from "node:url";
import {createHash} from "node:crypto";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const mode = process.argv[2] ?? "silent";

if (!["silent", "narrated"].includes(mode)) {
  throw new Error("Usage: node scripts/validate-assets.mjs [silent|narrated]");
}

const problems = [];
const productCaptures = [
  ["beta-v3-native-receipt.png", 405, 791],
  ["beta-v3-native-applied.png", 405, 791],
  ["beta-v3-native-apply-confirm.png", 440, 182],
  ["beta-v3-native-journal.png", 1024, 181],
];

const retiredSurfacePattern = /\b(?:Flow|Operations|Workspaces|Integrations)\b/;
const compositionSource = await readFile(
  path.join(appRoot, "src", "PytxoLaunchDemo.tsx"),
  "utf8",
);
if (retiredSurfacePattern.test(compositionSource)) {
  problems.push(
    "src/PytxoLaunchDemo.tsx: retired Desktop destination appears in the release composition",
  );
}

const readPngDimensions = async (relativePath, expectedWidth, expectedHeight) => {
  const buffer = await readFile(path.join(appRoot, relativePath));
  if (buffer.toString("ascii", 1, 4) !== "PNG") {
    problems.push(`${relativePath}: expected a PNG file`);
    return;
  }
  const width = buffer.readUInt32BE(16);
  const height = buffer.readUInt32BE(20);
  if (width !== expectedWidth || height !== expectedHeight) {
    problems.push(`${relativePath}: expected original ${expectedWidth}x${expectedHeight}, received ${width}x${height}`);
  }
};

for (const [filename, width, height] of productCaptures) {
  const capture = `public/product/${filename}`;
  try {
    await readPngDimensions(capture, width, height);
    const original = await readFile(path.resolve(appRoot, "../../docs/_attachments/beta-2026-09-06", filename));
    const copy = await readFile(path.join(appRoot, capture));
    const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
    if (hash(original) !== hash(copy)) problems.push(`${capture}: differs from the recorded native capture`);
  } catch {
    problems.push(`${capture}: missing original or copied native capture`);
  }
}

const probeAudio = (relativePath) => {
  const result = spawnSync(
    process.execPath,
    [
      path.join(appRoot, "node_modules", "@remotion", "cli", "remotion-cli.js"),
      "ffprobe",
      "-v",
      "error",
      "-show_entries",
      "format=duration:stream=codec_type,codec_name,sample_rate",
      "-of",
      "json",
      relativePath,
    ],
    {
      cwd: appRoot,
      encoding: "utf8",
    },
  );
  if (result.status !== 0) {
    problems.push(`${relativePath}: ffprobe could not read an audio stream`);
    return null;
  }
  const data = JSON.parse(result.stdout);
  if (!data.streams?.some((stream) => stream.codec_type === "audio")) {
    problems.push(`${relativePath}: no audio stream`);
    return null;
  }
  return Number(data.format?.duration);
};

if (mode === "narrated") {
  const requiredAudio = [
    "public/audio/narration/pytxo-demo-narration.mp3",
    "public/audio/music/modern-chillout-future-calm.mp3",
  ];
  const available = new Set();

  for (const relativePath of requiredAudio) {
    try {
      const metadata = await stat(path.join(appRoot, relativePath));
      if (metadata.size < 1024) {
        problems.push(`${relativePath}: file is too small to be an approved audio asset`);
      } else {
        available.add(relativePath);
      }
    } catch {
      problems.push(`${relativePath}: required for narrated master`);
    }
  }

  const certificatePath =
    "private-licenses/modern-chillout-future-calm-license.txt";
  try {
    const certificate = await readFile(path.join(appRoot, certificatePath), "utf8");
    if (
      !certificate.includes(
        "https://pixabay.com/music/upbeat-penguinmusic-modern-chillout-future-calm-12641/",
      )
    ) {
      problems.push(`${certificatePath}: certificate does not identify the approved source URL`);
    }
  } catch {
    problems.push(`${certificatePath}: Content ID download certificate is required`);
  }

  for (const relativePath of available) {
    const duration = probeAudio(relativePath);
    if (!Number.isFinite(duration)) continue;
    if (relativePath.includes("/narration/") && (duration < 40 || duration > 52.2)) {
      problems.push(
        `${relativePath}: continuous narration must be 40.0–52.2 seconds, received ${duration.toFixed(3)}`,
      );
    }
    if (relativePath.includes("/music/") && duration < 52) {
      problems.push(`${relativePath}: music bed must cover the full 52-second master`);
    }
  }
}

if (problems.length > 0) {
  process.stderr.write(
    `Asset validation failed for ${mode} master:\n${problems.map((item) => `- ${item}`).join("\n")}\n`,
  );
  process.exit(1);
}

process.stdout.write(`Asset validation passed for ${mode} master.\n`);
