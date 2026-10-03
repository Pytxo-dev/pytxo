// Small CDP probe for the upgrade check: records what the running Desktop shows,
// optionally leaves a WebView storage marker, and walks the guided-example
// onboarding when it is offered.
//
//   node tooling/acceptance/probe.mjs --cdp 9340 --out <dir> --name <label> [--set-marker <value>] [--onboard]
import { mkdirSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(path.join(import.meta.dirname, "../../apps/desktop/package.json"));
const { chromium } = require("@playwright/test");
const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1] === undefined || all[index + 1].startsWith("--") ? "true" : all[index + 1]]] : pairs), []));
const out = path.resolve(args.out);
mkdirSync(out, { recursive: true });

let browser;
for (let attempt = 0; attempt < 90 && !browser; attempt++) {
  try { browser = await chromium.connectOverCDP(`http://127.0.0.1:${args.cdp}`); } catch { await new Promise((resolve) => setTimeout(resolve, 1000)); }
}
if (!browser) throw new Error("Desktop never exposed its WebView over CDP");
const page = browser.contexts().flatMap((context) => context.pages())[0];
await page.waitForLoadState("domcontentloaded");
await page.waitForTimeout(4000);
const report = { name: args.name, url: page.url(), markerBefore: await page.evaluate(() => localStorage.getItem("pytxo-acceptance-marker")) };
if (args["set-marker"]) await page.evaluate((value) => localStorage.setItem("pytxo-acceptance-marker", value), args["set-marker"]);

if (args.onboard) {
  const click = async (name) => {
    const target = page.getByRole("button", { name, exact: true });
    if (await target.count()) { await target.first().click(); await page.waitForTimeout(2500); return true; }
    return false;
  };
  report.onboarding = [];
  for (const name of ["Get started", "Continue", "Try the guided example", "Continue", "Enter Pytxo Desktop"]) report.onboarding.push({ name, clicked: await click(name) });
}
report.headings = await page.getByRole("heading").allInnerTexts();
report.text = (await page.locator("body").innerText()).slice(0, 2000);
await page.screenshot({ path: path.join(out, `${args.name}.png`) });
writeFileSync(path.join(out, `${args.name}.json`), `${JSON.stringify(report, null, 2)}\n`);
console.log(JSON.stringify({ name: report.name, markerBefore: report.markerBefore, headings: report.headings.slice(0, 6) }));
await browser.close();
