import {readFile} from "node:fs/promises";
import path from "node:path";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const normalize = (text) => text.replace(/\s+/g, " ").trim();

const voiceover = await readFile(path.join(appRoot, "VOICEOVER.md"), "utf8");
const canonical = voiceover.match(
  /<!-- NARRATION_START -->\s*([\s\S]*?)\s*<!-- NARRATION_END -->/,
)?.[1];
if (!canonical) throw new Error("VOICEOVER.md is missing canonical narration markers");

const srt = await readFile(
  path.join(appRoot, "publishing", "pytxo-demo-en-provisional.srt"),
  "utf8",
);
const transcript = await readFile(
  path.join(appRoot, "publishing", "pytxo-demo-transcript.md"),
  "utf8",
);
const transcriptNarration = transcript
  .split(/\r?\n\r?\n/)
  .find((block) => !block.startsWith("#") && !block.startsWith("On-screen"));
const cues = srt
  .trim()
  .split(/\r?\n\r?\n/)
  .map((block) => block.split(/\r?\n/));
const errors = [];
const captionText = [];

for (const [index, lines] of cues.entries()) {
  if (lines.length < 3 || !lines[1].includes("-->")) {
    errors.push(`cue ${index + 1}: invalid SRT structure`);
    continue;
  }
  const textLines = lines.slice(2);
  if (textLines.length > 2) {
    errors.push(`cue ${index + 1}: expected no more than 2 readable text lines`);
  }
  captionText.push(...textLines);
}

if (normalize(captionText.join(" ")) !== normalize(canonical)) {
  errors.push("cue text does not concatenate to the exact canonical narration");
}
if (!transcriptNarration || normalize(transcriptNarration) !== normalize(canonical)) {
  errors.push("publishing transcript does not contain the exact canonical narration");
}
if (errors.length > 0) {
  throw new Error(`Publishing validation failed:\n${errors.map((error) => `- ${error}`).join("\n")}`);
}
process.stdout.write(
  `Publishing validation passed: transcript and ${cues.length} provisional cues contain the exact canonical narration; cues use 1–2 lines.\n`,
);
