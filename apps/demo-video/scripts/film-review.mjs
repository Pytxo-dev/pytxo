// Renders review stills of PytxoFilm from one bundle (fast, for iterating in CI).
//
//   node scripts/film-review.mjs [--every <seconds>] [--out out/film-review]
import {mkdirSync} from "node:fs";
import path from "node:path";
import {bundle} from "@remotion/bundler";
import {renderStill, selectComposition} from "@remotion/renderer";

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
const root = path.resolve(import.meta.dirname, "..");
const out = path.resolve(root, args.out ?? "out/film-review");
mkdirSync(out, {recursive: true});
const serveUrl = await bundle({entryPoint: path.join(root, "src/index.ts")});
const composition = await selectComposition({serveUrl, id: "PytxoFilm"});
const every = Math.round(Number(args.every ?? 2) * composition.fps);
for (let frame = 30; frame < composition.durationInFrames; frame += every) {
  const file = path.join(out, `${String(frame).padStart(5, "0")}.jpg`);
  await renderStill({serveUrl, composition, frame, output: file, imageFormat: "jpeg", jpegQuality: 85});
  console.log(file);
}
