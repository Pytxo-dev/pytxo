// The applied project, used as its own user would: served locally and opened in
// Edge (preinstalled on Windows runners) with the system dark preference, then
// switched to Spanish and given a task. Recorded frame by frame for the film.
//
//   node tooling/acceptance/result-app.mjs --out <dir> --repo <applied fixture>
//
// Writes result.png, result-frames/ with result-frames.txt (ffconcat, real timing)
// and result.json (what the page showed, film marks and pointer telemetry).
import { existsSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import http from "node:http";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(path.join(import.meta.dirname, "../../apps/desktop/package.json"));
const { chromium } = require("@playwright/test");
const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
const out = path.resolve(args.out);
const root = path.resolve(args.repo);
const report = { marks: {}, pointer: [] };

const types = { ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript", ".css": "text/css", ".json": "application/json" };
const server = http.createServer((request, response) => {
  const file = path.join(root, decodeURIComponent(new URL(request.url, "http://localhost").pathname));
  if (!file.startsWith(root) || !existsSync(file) || !statSync(file).isFile()) { response.writeHead(404).end(); return; }
  response.writeHead(200, { "content-type": types[path.extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
});
await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
const edge = await chromium.launch({ channel: "msedge" }).catch(() => chromium.launch());
const page = await edge.newPage({ viewport: { width: 1600, height: 1000 }, colorScheme: "dark" });

// Every compositor frame, with frame times in epoch milliseconds.
mkdirSync(path.join(out, "result-frames"), { recursive: true });
const frames = [];
const session = await page.context().newCDPSession(page);
session.on("Page.screencastFrame", ({ data, metadata, sessionId }) => {
  const file = `${String(frames.length).padStart(6, "0")}.jpg`;
  writeFileSync(path.join(out, "result-frames", file), Buffer.from(data, "base64"));
  frames.push({ file, t: metadata.timestamp * 1000 });
  session.send("Page.screencastFrameAck", { sessionId }).catch(() => {});
});
await session.send("Page.startScreencast", { format: "jpeg", quality: 92, everyNthFrame: 1 });

async function press(locator, label) {
  const box = await locator.boundingBox();
  if (box) report.pointer.push({ t: Date.now(), kind: "hover", label, x: Math.round(box.x + box.width / 2), y: Math.round(box.y + box.height / 2), surface: "result" });
  await locator.hover();
  await page.waitForTimeout(450);
  report.pointer.push({ t: Date.now(), kind: "click", label, surface: "result" });
  await locator.click();
}

// Live agents choose their own markup, so each control is found by what it does, and a
// feature that did not land is skipped rather than failing the recording.
const first = async (...selectors) => {
  for (const selector of selectors) {
    const locator = page.locator(selector).first();
    if (await locator.count().catch(() => 0)) return locator;
  }
  return null;
};

try {
  await page.goto(`http://127.0.0.1:${server.address().port}/index.html`);
  await page.locator("#tasks li").first().waitFor();
  report.marks.result = Date.now();
  await page.waitForTimeout(1500);
  const language = await first("#language-switch", "select:has(option[value=es])");
  const spanish = language ? null : await first("button:has-text('Español')", "button:has-text('ES')", "[data-lang=es]");
  if (language || spanish) {
    await press(language ?? spanish, "Language");
    if (language) await language.selectOption("es");
    await page.waitForFunction(() => document.documentElement.lang === "es", null, { timeout: 5000 }).catch(() => {});
    await page.waitForTimeout(1200);
  }
  const es = await page.evaluate(() => document.documentElement.lang === "es");
  const input = await first("#new-task input", "form input[type=text]", "form input:not([type])");
  if (input) {
    await press(input, "New task");
    await input.pressSequentially(es ? "Publicar la versión beta" : "Ship the beta", { delay: 45 });
    const add = await first("#new-task button", "form button");
    if (add) await press(add, "Add"); else await input.press("Enter");
    await page.waitForTimeout(800);
  }
  const done = page.locator("#tasks input[type=checkbox]").last();
  if (await done.count()) await press(done, "Done");
  await page.waitForTimeout(1500);
  await page.screenshot({ path: path.join(out, "result.png") });
  report.lang = await page.evaluate(() => document.documentElement.lang);
  report.tasks = await page.locator("#tasks li").allInnerTexts();
} finally {
  await session.send("Page.stopScreencast").catch(() => {});
  if (frames.length) {
    const entry = (frame) => `file 'result-frames/${frame.file}'`;
    const list = frames.map((frame, index) => `${entry(frame)}\nduration ${(((frames[index + 1]?.t ?? frame.t + 1000) - frame.t) / 1000).toFixed(3)}`);
    writeFileSync(path.join(out, "result-frames.txt"), ["ffconcat version 1.0", ...list, entry(frames.at(-1)), ""].join("\n"));
    report.first = frames[0].t;
    report.frames = frames.length;
  }
  writeFileSync(path.join(out, "result.json"), `${JSON.stringify(report, null, 2)}\n`);
  await edge.close();
  server.close();
}
console.log(JSON.stringify({ lang: report.lang, tasks: report.tasks?.length, frames: report.frames }));
