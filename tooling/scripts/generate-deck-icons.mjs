#!/usr/bin/env node
/**
 * Generate the Tauri icon set and the standalone `logo-mark.png` from
 * apps/web/public/logo.png.
 *
 * The source logo is a 1024x1024 canvas with large transparent padding
 * around the lambda glyph. Icons are built by trimming that padding first,
 * then compositing the tight glyph onto a fully transparent canvas at each
 * target size. This keeps every icon alpha-transparent (so it reads
 * correctly against dark/light taskbars and docks) and keeps the glyph
 * optically sized close to sibling app icons instead of shrinking it inside
 * unused padding.
 */
import fs from "node:fs";
import path from "node:path";
import { execSync } from "node:child_process";
import { fileURLToPath } from "node:url";

import sharp from "sharp";
import toIco from "to-ico";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const logoPath = path.join(root, "apps/web/public/logo.png");
const logoMarkPath = path.join(root, "apps/web/public/logo-mark.png");
const iconsDir = path.join(root, "apps/desktop/src-tauri/icons");

/** Glyph fills this fraction of the icon canvas's longer trimmed dimension. */
const FILL_RATIO = 0.86;
/** Minimum fraction of canvas the glyph bounding box must cover (regression guard). */
const MIN_GLYPH_COVERAGE = 0.55;

/** Trim the source logo's transparent padding down to the tight glyph bounds. */
async function trimmedGlyph() {
  const { data, info } = await sharp(logoPath).trim({ threshold: 10 }).toBuffer({ resolveWithObject: true });
  return { buffer: data, width: info.width, height: info.height };
}

/** Composite the trimmed glyph centered on a fully transparent size x size canvas. */
async function iconPng(glyph, size) {
  const target = Math.round(size * FILL_RATIO);
  const scale = Math.min(target / glyph.width, target / glyph.height);
  const w = Math.max(1, Math.round(glyph.width * scale));
  const h = Math.max(1, Math.round(glyph.height * scale));
  const left = Math.round((size - w) / 2);
  const top = Math.round((size - h) / 2);

  const resized = await sharp(glyph.buffer).resize(w, h, { fit: "inside" }).png().toBuffer();
  return sharp({
    create: { width: size, height: size, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } },
  })
    .composite([{ input: resized, left, top }])
    .png()
    .toBuffer();
}

/** Fail loudly if an icon regresses to opaque corners or an undersized glyph. */
async function assertIconQuality(buffer, size, label) {
  const raw = await sharp(buffer).raw().toBuffer({ resolveWithObject: true });
  const { data, info } = raw;
  const channels = info.channels;
  const corners = [
    [0, 0],
    [info.width - 1, 0],
    [0, info.height - 1],
    [info.width - 1, info.height - 1],
  ];
  for (const [x, y] of corners) {
    const idx = (y * info.width + x) * channels + (channels - 1);
    if (data[idx] !== 0) {
      throw new Error(`${label}: corner (${x},${y}) is not transparent (alpha=${data[idx]})`);
    }
  }

  let minX = info.width;
  let minY = info.height;
  let maxX = -1;
  let maxY = -1;
  for (let y = 0; y < info.height; y += 1) {
    for (let x = 0; x < info.width; x += 1) {
      const a = data[(y * info.width + x) * channels + (channels - 1)];
      if (a > 10) {
        if (x < minX) minX = x;
        if (x > maxX) maxX = x;
        if (y < minY) minY = y;
        if (y > maxY) maxY = y;
      }
    }
  }
  const coverage = Math.max((maxX - minX + 1) / size, (maxY - minY + 1) / size);
  if (coverage < MIN_GLYPH_COVERAGE) {
    throw new Error(`${label}: glyph only covers ${(coverage * 100).toFixed(0)}% of canvas (expected >= ${MIN_GLYPH_COVERAGE * 100}%)`);
  }
}

async function writeIcns(pngBuffers, glyph, outPath) {
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
      const buf = pngBuffers.get(size) ?? (await iconPng(glyph, size));
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
  const glyph = await trimmedGlyph();

  const sizes = [32, 128, 256];
  for (const size of sizes) {
    const name = size === 256 ? "128x128@2x.png" : `${size}x${size}.png`;
    const buf = await iconPng(glyph, size);
    await assertIconQuality(buf, size, name);
    fs.writeFileSync(path.join(iconsDir, name), buf);
  }

  const png256 = await iconPng(glyph, 256);
  const png128 = await iconPng(glyph, 128);
  const png64 = await iconPng(glyph, 64);
  const png48 = await iconPng(glyph, 48);
  const png32 = await iconPng(glyph, 32);
  const png16 = await iconPng(glyph, 16);

  for (const [buf, size, label] of [
    [png256, 256, "icon.png (256)"],
    [png128, 128, "ico:128"],
    [png64, 64, "ico:64"],
    [png48, 48, "ico:48"],
    [png32, 32, "ico:32"],
    [png16, 16, "ico:16"],
  ]) {
    await assertIconQuality(buf, size, label);
  }

  const ico = await toIco([png16, png32, png48, png64, png128, png256]);
  fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);
  fs.writeFileSync(path.join(iconsDir, "icon.png"), png256);

  const icnsCache = new Map([
    [16, png16],
    [32, png32],
    [64, png64],
    [128, png128],
    [256, png256],
    [512, await iconPng(glyph, 512)],
    [1024, await iconPng(glyph, 1024)],
  ]);
  await writeIcns(icnsCache, glyph, path.join(iconsDir, "icon.icns"));

  // Standalone transparent mark for in-app chrome (title bar, sidebar, setup wizard).
  const mark = await iconPng(glyph, 512);
  await assertIconQuality(mark, 512, "logo-mark.png");
  fs.writeFileSync(logoMarkPath, mark);

  console.log(`generate-deck-icons: wrote icons to ${iconsDir}`);
  console.log(`generate-deck-icons: wrote ${logoMarkPath}`);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
