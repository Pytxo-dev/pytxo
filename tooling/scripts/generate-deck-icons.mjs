#!/usr/bin/env node
/**
 * Generate Tauri icon set from apps/web/public/logo.png (Chroma void background).
 * Logo fills ~88% of canvas for full-sized taskbar/dock appearance.
 */
import fs from "node:fs";
import path from "node:path";
import { execSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import sharp from "sharp";
import toIco from "to-ico";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const logoPath = path.join(root, "apps/web/public/logo.png");
const iconsDir = path.join(root, "apps/desktop/src-tauri/icons");

const FILL_RATIO = 0.88;
const bg = { r: 2, g: 2, b: 5, alpha: 1 };

/** Composite logo centered at FILL_RATIO of canvas size. */
async function iconPng(logo, size) {
  const meta = await logo.metadata();
  const lw = meta.width ?? 512;
  const lh = meta.height ?? 512;
  const target = Math.round(size * FILL_RATIO);
  const scale = Math.min(target / lw, target / lh);
  const w = Math.round(lw * scale);
  const h = Math.round(lh * scale);
  const left = Math.round((size - w) / 2);
  const top = Math.round((size - h) / 2);

  const resized = await logo.clone().resize(w, h, { fit: "inside" }).png().toBuffer();
  return sharp({
    create: { width: size, height: size, channels: 4, background: bg },
  })
    .composite([{ input: resized, left, top }])
    .png()
    .toBuffer();
}

async function writeIcns(pngBuffers, outPath) {
  const tmp = fs.mkdtempSync(path.join(root, ".iconset-"));
  try {
    const icnsSizes = [
      [16, "icon_16x16.png"],
      [32, "icon_16x16@2x.png"],
      [32, "icon_32x32.png"],
      [64, "icon_32x32@2x.png"],
      [128, "icon_128x128.png"],
      [256, "icon_128x128@2x.png"],
      [256, "icon_256x256.png"],
      [512, "icon_256x256@2x.png"],
      [512, "icon_512x512.png"],
      [1024, "icon_512x512@2x.png"],
    ];
    for (const [size, name] of icnsSizes) {
      const buf = pngBuffers.get(size) ?? (await iconPng(sharp(logoPath), size));
      fs.writeFileSync(path.join(tmp, name), buf);
    }
    const iconset = `${tmp}.iconset`;
    fs.renameSync(tmp, iconset);
    try {
      execSync(`iconutil -c icns "${iconset}" -o "${outPath}"`, { stdio: "pipe" });
    } finally {
      fs.rmSync(iconset, { recursive: true, force: true });
    }
  } catch (e) {
    fs.rmSync(tmp, { recursive: true, force: true });
    console.warn("generate-deck-icons: icon.icns skipped (iconutil unavailable on this OS)");
  }
}

async function main() {
  if (!fs.existsSync(logoPath)) {
    throw new Error(`logo not found: ${logoPath}`);
  }
  fs.mkdirSync(iconsDir, { recursive: true });
  const logo = sharp(logoPath);

  const sizes = [32, 128, 256];
  for (const size of sizes) {
    const name = size === 256 ? "128x128@2x.png" : `${size}x${size}.png`;
    const buf = await iconPng(logo, size);
    fs.writeFileSync(path.join(iconsDir, name), buf);
  }

  const png256 = await iconPng(logo, 256);
  const png128 = await iconPng(logo, 128);
  const png64 = await iconPng(logo, 64);
  const png48 = await iconPng(logo, 48);
  const png32 = await iconPng(logo, 32);
  const png16 = await iconPng(logo, 16);

  const ico = await toIco([png16, png32, png48, png64, png128, png256]);
  fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);
  fs.writeFileSync(path.join(iconsDir, "icon.png"), png256);

  const icnsCache = new Map([
    [16, png16],
    [32, png32],
    [64, png64],
    [128, png128],
    [256, png256],
    [512, await iconPng(logo, 512)],
    [1024, await iconPng(logo, 1024)],
  ]);
  await writeIcns(icnsCache, path.join(iconsDir, "icon.icns"));

  console.log(`generate-deck-icons: wrote icons to ${iconsDir}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
