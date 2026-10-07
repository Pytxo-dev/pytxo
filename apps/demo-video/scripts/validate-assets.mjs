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
const evidence = JSON.parse(await readFile(path.resolve(appRoot, "../../tooling/benchmarks/results/astra-final-native-2026-09-08.json"), "utf8"));
const requiredCaptures = ["astra-final-native-plan.png", "astra-final-native-checks.png", "astra-final-native-boundaries.png", "astra-final-native-apply-confirm.png", "astra-final-native-applied.png", "astra-final-native-journal.png"].sort();
const productCaptures = evidence.native_apply.captures;
if (JSON.stringify(productCaptures.map((capture) => capture.file).sort()) !== JSON.stringify(requiredCaptures)) {
  throw new Error("Native evidence must identify all six original captures exactly once.");
}
const postCheck = evidence.native_apply.post_apply_check;
if (postCheck?.command !== "npm test" || postCheck.exit_code !== 0 || postCheck.failed !== 0 || postCheck.skipped !== 0 || !Number.isInteger(postCheck.passed) || postCheck.passed !== evidence.native_apply.post_apply_tests_passed || postCheck.passed <= 2) {
  throw new Error("The film requires a recorded passing independent post-Apply test process.");
}
if (evidence.run_status !== "completed" || !evidence.native_apply.applied_files_match_frozen_digests || evidence.changed_files.length !== 3 || evidence.planned_tasks !== 3 || evidence.waves !== 2 || evidence.max_concurrent_workers !== 2) {
  throw new Error("The film's outcome does not match the recorded native run.");
}
const independent = evidence.native_apply.independent_acceptance;
if (!evidence.native_apply.persisted_after_native_restart || !independent?.authored_before_apply || independent.baseline_failed !== 5 || independent.post_apply_passed !== 26 || independent.post_apply_failed !== 0 || independent.post_apply_skipped !== 0) {
  throw new Error("The film requires the separate red/green acceptance check and persisted native receipt.");
}

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

for (const {file: filename, width, height, sha256} of productCaptures) {
  const capture = `public/product/${filename}`;
  try {
    await readPngDimensions(capture, width, height);
    const original = await readFile(path.resolve(appRoot, "../../docs/_attachments/astra-2026-09-08", filename));
    const copy = await readFile(path.join(appRoot, capture));
    const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
    if (hash(original) !== hash(copy) || hash(original) !== sha256) problems.push(`${capture}: differs from the recorded native capture`);
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
  try {
    const document = await readFile(path.join(appRoot, "VOICEOVER.md"), "utf8");
    const script = document.match(/<!-- NARRATION_START -->\s*([\s\S]*?)\s*<!-- NARRATION_END -->/)?.[1]?.replace(/\s+/g, " ").trim();
    const identity = JSON.parse(await readFile(path.join(appRoot, "public/audio/narration/identity.json"), "utf8"));
    const audio = await readFile(path.join(appRoot, "public/audio/narration/pytxo-demo-narration.mp3"));
    const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
    if (!script || identity.script_sha256 !== hash(script) || identity.audio_sha256 !== hash(audio)) problems.push("Narration does not match the current script/audio identity; regenerate and review it.");
  } catch {
    problems.push("Current narration identity is missing; older audio cannot certify this edit.");
  }
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
