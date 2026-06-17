#!/usr/bin/env node
/**
 * Generate Tauri icon set from apps/web/public/logo.png (Chroma void background).
 */
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import sharp from "sharp";
import toIco from "to-ico";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const logoPath = path.join(root, "apps/web/public/logo.png");
const iconsDir = path.join(root, "apps/desktop/src-tauri/icons");

const sizes = [32, 128, 256];

async function main() {
  if (!fs.existsSync(logoPath)) {
    throw new Error(`logo not found: ${logoPath}`);
  }
  fs.mkdirSync(iconsDir, { recursive: true });
  const logo = sharp(logoPath);
  const bg = "#020205";

  for (const size of sizes) {
    const name = size === 256 ? "128x128@2x.png" : `${size}x${size}.png`;
    await logo
      .clone()
      .resize(size, size, { fit: "contain", background: bg })
      .png()
      .toFile(path.join(iconsDir, name));
  }

  const png256 = await logo.clone().resize(256, 256, { fit: "contain", background: bg }).png().toBuffer();
  const png128 = await logo.clone().resize(128, 128, { fit: "contain", background: bg }).png().toBuffer();
  const png64 = await logo.clone().resize(64, 64, { fit: "contain", background: bg }).png().toBuffer();
  const png48 = await logo.clone().resize(48, 48, { fit: "contain", background: bg }).png().toBuffer();
  const png32 = await logo.clone().resize(32, 32, { fit: "contain", background: bg }).png().toBuffer();
  const png16 = await logo.clone().resize(16, 16, { fit: "contain", background: bg }).png().toBuffer();

  const ico = await toIco([png16, png32, png48, png64, png128, png256]);
  fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);

  // icon.png root for tauri bundle
  fs.writeFileSync(path.join(iconsDir, "icon.png"), png256);

  console.log(`generate-deck-icons: wrote icons to ${iconsDir}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
