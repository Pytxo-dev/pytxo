import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import sharp from "sharp";
import toIco from "to-ico";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const logoPath = path.join(root, "public/logo.png");
const appDir = path.join(root, "src/app");
const publicDir = path.join(root, "public");

async function main() {
  const logo = sharp(logoPath);

  await logo.clone().resize(32, 32, { fit: "contain", background: "#020205" }).png().toFile(path.join(appDir, "icon.png"));

  await logo
    .clone()
    .resize(180, 180, { fit: "contain", background: "#020205" })
    .png()
    .toFile(path.join(appDir, "apple-icon.png"));

  const png16 = await logo.clone().resize(16, 16, { fit: "contain", background: "#020205" }).png().toBuffer();
  const png32 = await logo.clone().resize(32, 32, { fit: "contain", background: "#020205" }).png().toBuffer();
  const png48 = await logo.clone().resize(48, 48, { fit: "contain", background: "#020205" }).png().toBuffer();

  const ico = await toIco([png16, png32, png48]);
  fs.writeFileSync(path.join(appDir, "favicon.ico"), ico);
  fs.writeFileSync(path.join(publicDir, "favicon.ico"), ico);

  console.log("generate-favicon: wrote icon.png, apple-icon.png, favicon.ico");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
