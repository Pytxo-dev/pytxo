import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import sharp from "sharp";
import { encodePngIco } from "../../../tooling/scripts/encode-png-ico.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const logoDir = path.join(root, "vendor/chroma/logo");
const appDir = path.join(root, "src/app");
const publicDir = path.join(root, "public");

/**
 * The mark's λ is ink, so a favicon has to bring its own Void tile to read on a
 * light tab. 16 px uses the tile drawn on that pixel grid. librsvg renders each
 * size directly, which keeps the rounded corners transparent.
 */
function tile(size) {
  const [file, units] = size <= 16 ? ["pytxo-icon-16.svg", 16] : ["pytxo-icon.svg", 480];
  return sharp(path.join(logoDir, file), { density: (72 * size) / units })
    .resize(size, size)
    .png();
}

async function main() {
  await tile(32).toFile(path.join(appDir, "icon.png"));
  await tile(180).toFile(path.join(appDir, "apple-icon.png"));

  const ico = encodePngIco(await Promise.all([16, 32, 48].map((size) => tile(size).toBuffer())));
  fs.writeFileSync(path.join(appDir, "favicon.ico"), ico);
  fs.writeFileSync(path.join(publicDir, "favicon.ico"), ico);

  console.log("generate-favicon: wrote icon.png, apple-icon.png, favicon.ico");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
