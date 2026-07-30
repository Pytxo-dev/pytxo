import {mkdir, readFile, writeFile} from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import {fileURLToPath} from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const appRoot = path.resolve(here, "..");
const scenes = JSON.parse(
  await readFile(path.join(appRoot, "src", "voiceover.json"), "utf8"),
);

const apiKey = process.env.ELEVENLABS_API_KEY;
const voiceId = process.env.ELEVENLABS_VOICE_ID;
const modelId =
  process.env.ELEVENLABS_MODEL_ID ?? "eleven_multilingual_v2";

if (!apiKey || !voiceId) {
  throw new Error(
    "Set ELEVENLABS_API_KEY and ELEVENLABS_VOICE_ID before generating voiceover.",
  );
}

const outputDir = path.join(appRoot, "public", "voiceover");
await mkdir(outputDir, {recursive: true});

for (const scene of scenes) {
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
        text: scene.voiceLine,
        model_id: modelId,
        voice_settings: {
          stability: 0.58,
          similarity_boost: 0.78,
          style: 0.18,
          use_speaker_boost: true,
        },
      }),
    },
  );

  if (!response.ok) {
    throw new Error(
      `ElevenLabs failed for ${scene.id}: ${response.status} ${await response.text()}`,
    );
  }

  const target = path.join(outputDir, `${scene.id}.mp3`);
  await writeFile(target, Buffer.from(await response.arrayBuffer()));
  process.stdout.write(`generated ${path.relative(appRoot, target)}\n`);
}
