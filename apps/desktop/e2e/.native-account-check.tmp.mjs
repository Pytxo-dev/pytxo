// Temporary native check (not committed): drives the release exe over WebView2 CDP.
import { chromium } from "@playwright/test";

const out = process.argv[2];
const browser = await chromium.connectOverCDP("http://127.0.0.1:9333");
const context = browser.contexts()[0];
let page = context.pages().find((p) => !p.url().startsWith("devtools"));
for (let i = 0; !page && i < 40; i++) {
  await new Promise((r) => setTimeout(r, 250));
  page = context.pages().find((p) => !p.url().startsWith("devtools"));
}
const errors = [];
page.on("console", (m) => m.type() === "error" && errors.push(m.text()));
await page.evaluate(() => {
  for (const key of Object.keys(localStorage)) if (key.includes("setup") || key.includes("onboarding")) localStorage.setItem(key, "complete");
});
await page.waitForTimeout(800);
const version = await page.evaluate(async () => {
  const ipc = window.__TAURI_INTERNALS__;
  return ipc ? await ipc.invoke("account_status") : null;
});
console.log("account_status", JSON.stringify(version));
await page.screenshot({ path: `${out}/native-start.png` });
console.log("url", page.url());
console.log("errors", JSON.stringify(errors));
await browser.close();
