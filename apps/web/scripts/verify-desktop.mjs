// Temporary verification: render the new Work/History/Setup surfaces and report what is on screen.
import { chromium } from "@playwright/test";
import { mkdir } from "node:fs/promises";

const BASE = process.argv[2] ?? "http://127.0.0.1:5199";
const OUT = "scripts/.verify";

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1600, height: 1000 } });
await page.emulateMedia({ reducedMotion: "reduce" });
await page.addInitScript(() => {
  localStorage.setItem("pytxo-deck-setup-v1", "complete");
  localStorage.setItem("pytxo-desktop-onboarding-version", "1.1.0");
});
await mkdir(OUT, { recursive: true });

const errors = [];
page.on("console", (m) => {
  if (m.type() === "error") errors.push(m.text());
});
page.on("pageerror", (e) => errors.push(String(e)));

async function go(route) {
  await page.goto(`${BASE}/#/${route}`, { waitUntil: "domcontentloaded" });
  await page.waitForTimeout(2200);
}

async function report(name) {
  const info = await page.evaluate(() => ({
    h1: [...document.querySelectorAll("h1")].map((n) => n.textContent?.trim()).slice(0, 2),
    h2: [...document.querySelectorAll("h2")].map((n) => n.textContent?.trim()).slice(0, 5),
    h3: [...document.querySelectorAll("h3")].map((n) => n.textContent?.trim()).slice(0, 5),
    dialogs: [...document.querySelectorAll("dialog[open]")].map((n) => n.getAttribute("aria-label") ?? "dialog"),
    overflowX: document.documentElement.scrollWidth - document.documentElement.clientWidth,
  }));
  await page.screenshot({ path: `${OUT}/${name}.png`, animations: "disabled" });
  console.log(name.padEnd(18), JSON.stringify(info));
}

for (const route of ["work", "history", "setup", "approvals"]) {
  await go(route);
  await report(route);
}

// Selecting a ledger row must reveal the per-agent enforcement receipt inline.
await go("work");
await page.locator(".row").first().click();
await page.waitForTimeout(500);
await report("work-agent-selected");
const inspector = await page.locator("text=Per-agent enforcement").count();
console.log("inspector sections:", inspector);

console.log("\nconsole errors:", errors.length ? [...new Set(errors)] : "none");
await browser.close();
