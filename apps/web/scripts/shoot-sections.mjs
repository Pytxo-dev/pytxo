import { mkdir } from "node:fs/promises";
import { chromium } from "@playwright/test";

const BASE = process.env.SHOOT_BASE ?? "http://127.0.0.1:3119";
const OUT = "captures/site";

const SECTIONS = [
  "marketing-hero",
  "product-section",
  "boundary-section",
  "compatibility-section",
  "evidence-section",
  "get-it-section",
];

await mkdir(OUT, { recursive: true });

const browser = await chromium.launch();
const context = await browser.newContext({
  viewport: { width: 1440, height: 900 },
  deviceScaleFactor: 1,
});
const page = await context.newPage();
await page.goto(`${BASE}/`, { waitUntil: "load" });
await page.waitForTimeout(600);

// The site header is sticky, so it lands on top of any section screenshot taken
// below the fold. Hide it: these shots are for reading section composition.
await page.addStyleTag({ content: "header[data-site-header]{visibility:hidden}" });

for (const id of SECTIONS) {
  await page.locator(`[data-testid="${id}"]`).screenshot({ path: `${OUT}/section-${id}.png` });
  console.log(`shot ${id}`);
}

await page.locator("section", { has: page.locator("#situation-title") }).first()
  .screenshot({ path: `${OUT}/section-situation.png` });
console.log("shot situation");

for (const route of ["/evidence", "/download"]) {
  await page.goto(`${BASE}${route}`, { waitUntil: "load" });
  await page.waitForTimeout(400);
  await page.screenshot({ path: `${OUT}/page-${route.slice(1)}.png`, fullPage: true });
  console.log(`shot ${route}`);
}

await context.close();
await browser.close();
