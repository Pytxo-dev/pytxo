import { mkdir } from "node:fs/promises";
import { chromium } from "@playwright/test";

const BASE = process.env.SHOOT_BASE ?? "http://127.0.0.1:3114";
const OUT = "captures/site";

const SHOTS = [
  { name: "home-1920", path: "/", width: 1920, height: 1080, full: false },
  { name: "home-1440", path: "/", width: 1440, height: 900, full: false },
  { name: "home-full-1440", path: "/", width: 1440, height: 900, full: true },
  { name: "home-390", path: "/", width: 390, height: 844, full: false },
  { name: "home-full-390", path: "/", width: 390, height: 844, full: true },
  { name: "evidence-1440", path: "/evidence", width: 1440, height: 900, full: true },
  { name: "download-1440", path: "/download", width: 1440, height: 900, full: true },
  { name: "docs-1440", path: "/docs", width: 1440, height: 900, full: false },
];

await mkdir(OUT, { recursive: true });

const browser = await chromium.launch();
const problems = [];

for (const shot of SHOTS) {
  const context = await browser.newContext({
    viewport: { width: shot.width, height: shot.height },
    deviceScaleFactor: 1,
  });
  const page = await context.newPage();
  page.on("console", (message) => {
    if (message.type() === "error") problems.push(`${shot.name}: console ${message.text()}`);
  });
  page.on("pageerror", (error) => problems.push(`${shot.name}: pageerror ${error.message}`));

  await page.goto(`${BASE}${shot.path}`, { waitUntil: "load" });
  await page.waitForTimeout(900);

  const overflow = await page.evaluate(() => {
    const main = document.querySelector("main");
    return {
      document: document.documentElement.scrollWidth - document.documentElement.clientWidth,
      main: main ? main.scrollWidth - main.clientWidth : 0,
    };
  });
  if (overflow.document > 1 || overflow.main > 1) {
    problems.push(`${shot.name}: horizontal overflow ${JSON.stringify(overflow)}`);
  }

  await page.screenshot({ path: `${OUT}/${shot.name}.png`, fullPage: shot.full });
  await context.close();
  console.log(`shot ${shot.name}`);
}

await browser.close();

if (problems.length > 0) {
  console.log("\nPROBLEMS:");
  for (const problem of [...new Set(problems)]) console.log(` - ${problem}`);
} else {
  console.log("\nNo console errors and no horizontal overflow.");
}
