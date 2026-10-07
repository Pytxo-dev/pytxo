import {mkdir, readFile, writeFile} from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";
import {createHash} from "node:crypto";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const voiceoverDocument = await readFile(
  path.join(appRoot, "VOICEOVER.md"),
  "utf8",
);
const narrationMatch = voiceoverDocument.match(
  /<!-- NARRATION_START -->\s*([\s\S]*?)\s*<!-- NARRATION_END -->/,
);

if (!narrationMatch?.[1]) {
  throw new Error(
    "VOICEOVER.md must contain one paragraph between NARRATION_START and NARRATION_END.",
  );
}

const narration = narrationMatch[1].replace(/\s+/g, " ").trim();
if (narration.includes("\n") || narration.length < 100) {
  throw new Error("The canonical narration must be one non-empty paragraph.");
}

const apiKey = process.env.ELEVENLABS_API_KEY;
const voiceId = process.env.ELEVENLABS_VOICE_ID;
const modelId =
  process.env.ELEVENLABS_MODEL_ID ?? "eleven_multilingual_v2";

if (!apiKey || !voiceId) {
  throw new Error(
    "Narration generation blocked: set both ELEVENLABS_API_KEY and ELEVENLABS_VOICE_ID. No fallback provider or voice is permitted.",
  );
}

const outputDir = path.join(appRoot, "public", "audio", "narration");
await mkdir(outputDir, {recursive: true});

const response = await fetch(
  `https://api.elevenlabs.io/v1/text-to-speech/${voiceId}`,
  {
    method: "POST",
    headers: {
      "xi-api-key": apiKey,
      "Content-Type": "application/json",
      Accept: "audio/mpeg",
    },
    body: JSON.stringify({
      text: narration,
      model_id: modelId,
      voice_settings: {
        stability: 0.58,
        similarity_boost: 0.78,
        style: 0.18,
        speed: 0.86,
        use_speaker_boost: true,
      },
    }),
  },
);

if (!response.ok) {
  throw new Error(
    `ElevenLabs narration request failed: ${response.status} ${await response.text()}`,
  );
}

const target = path.join(outputDir, "pytxo-demo-narration.mp3");
const audio = Buffer.from(await response.arrayBuffer());
await writeFile(target, audio);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
await writeFile(path.join(outputDir, "identity.json"), JSON.stringify({script_sha256: hash(narration), audio_sha256: hash(audio)}, null, 2) + "\n");
process.stdout.write(`generated ${path.relative(appRoot, target)}\n`);
process.stdout.write(
  "Review the continuous track, then run `npm run render:narrated`; the asset gate will verify the full audio set.\n",
);
