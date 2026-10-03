// Drives an installed Pytxo Desktop over CDP for cloud acceptance.
//
//   node tooling/acceptance/journey.mjs --cdp 9340 --out <dir> --mode full --repo <fixture> --mission <file> [--team "Claude Code,…"]
//   node tooling/acceptance/journey.mjs --cdp 9340 --out <dir> --mode layout
//
// full:   onboarding → real folder dialog → mixed-agent plan → run → Review →
//         stale refusal → refresh → Apply, recording frames and a receipt.
// layout: onboarding with the guided example, then each destination checked for
//         clipped or overflowing layout at the launched device scale factor.
// Every step is logged; any failure leaves a screenshot and the page text.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, statSync, unlinkSync, writeFileSync } from "node:fs";
import http from "node:http";
import { createRequire } from "node:module";
import path from "node:path";

const here = import.meta.dirname;
const require = createRequire(path.join(here, "../../apps/desktop/package.json"));
const { chromium } = require("@playwright/test");

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
const out = path.resolve(args.out);
mkdirSync(out, { recursive: true });
const started = Date.now();
const receipt = { mode: args.mode, started: new Date(started).toISOString(), steps: [], checks: {} };
const step = (label, extra = {}) => { receipt.steps.push({ t: Date.now() - started, label, ...extra }); console.log(`${((Date.now() - started) / 1000).toFixed(1)}s ${label}`); };
const save = () => writeFileSync(path.join(out, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);

let browser;
for (let attempt = 0; attempt < 60 && !browser; attempt++) {
  try { browser = await chromium.connectOverCDP(`http://127.0.0.1:${args.cdp}`); } catch { await new Promise((resolve) => setTimeout(resolve, 1000)); }
}
if (!browser) throw new Error("Desktop never exposed its WebView over CDP");
const page = browser.contexts().flatMap((context) => context.pages())[0];
receipt.viewport = await page.evaluate(() => ({ width: innerWidth, height: innerHeight, devicePixelRatio }));
step("connected", receipt.viewport);

// Continuous capture: every compositor frame of a page, with an ffconcat list
// that keeps real timing (frame times are wall-clock epoch milliseconds).
async function record(target, dir) {
  mkdirSync(path.join(out, dir), { recursive: true });
  const frames = [];
  const session = await target.context().newCDPSession(target);
  session.on("Page.screencastFrame", ({ data, metadata, sessionId }) => {
    const file = `${String(frames.length).padStart(6, "0")}.jpg`;
    writeFileSync(path.join(out, dir, file), Buffer.from(data, "base64"));
    frames.push({ file, t: metadata.timestamp * 1000 });
    session.send("Page.screencastFrameAck", { sessionId }).catch(() => {});
  });
  await session.send("Page.startScreencast", { format: "jpeg", quality: 92, everyNthFrame: 1 });
  return async () => {
    await session.send("Page.stopScreencast").catch(() => {});
    if (!frames.length) return 0;
    const entry = (frame) => `file '${dir}/${frame.file}'`;
    const list = frames.map((frame, index) => `${entry(frame)}\nduration ${(((frames[index + 1]?.t ?? frame.t + 1000) - frame.t) / 1000).toFixed(3)}`);
    writeFileSync(path.join(out, `${dir}.txt`), ["ffconcat version 1.0", ...list, entry(frames.at(-1)), ""].join("\n"));
    writeFileSync(path.join(out, `${dir}.json`), `${JSON.stringify({ first: frames[0].t, last: frames.at(-1).t, count: frames.length })}\n`);
    return frames.length;
  };
}
const stopRecording = args.mode === "full" ? await record(page, "frames") : async () => 0;

const shot = (name) => page.screenshot({ path: path.join(out, `${name}.png`) });
/** Layout must fit: no horizontal page overflow, and named controls fully on screen. */
async function fits(name, controls = []) {
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  const offscreen = [];
  for (const control of controls) if (!(await control.isVisible()) || !(await isInViewport(control))) offscreen.push(String(control));
  receipt.checks[name] = { overflow, offscreen };
  await shot(name);
  if (overflow > 1 || offscreen.length) throw new Error(`${name} does not fit: overflow ${overflow}px, off screen ${offscreen.join(", ")}`);
}
async function isInViewport(locator) {
  const box = await locator.boundingBox();
  const { width, height } = page.viewportSize() ?? (await page.evaluate(() => ({ width: innerWidth, height: innerHeight })));
  return !!box && box.x >= -1 && box.y >= -1 && box.x + box.width <= width + 1 && box.y + box.height <= height + 1;
}
const button = (name, exact = true) => page.getByRole("button", { name, exact });
// Pointer telemetry for the film: hover, settle, then click, so hover feedback is
// on screen before each click. Times are epoch milliseconds, like frame times.
receipt.pointer = [];
async function press(locator, label, target = page) {
  const surface = target === page ? "desktop" : "result";
  await locator.scrollIntoViewIfNeeded();
  const box = await locator.boundingBox();
  if (box) receipt.pointer.push({ t: Date.now(), kind: "hover", label, surface, x: Math.round(box.x + box.width / 2), y: Math.round(box.y + box.height / 2) });
  await locator.hover();
  await target.waitForTimeout(450);
  receipt.pointer.push({ t: Date.now(), kind: "click", label, surface });
  await locator.click();
}
// Epoch-timed marks the film cuts on, alongside the acceptance steps.
receipt.marks = {};
const mark = (name) => { receipt.marks[name] = Date.now(); };

try {
  step("onboarding");
  await button("Get started").click();
  await page.getByText("Finding your coding agents").waitFor({ state: "detached", timeout: 90_000 }).catch(() => {});
  await page.waitForTimeout(1000);
  await fits("onboarding-agents", [button("Continue")]);
  receipt.checks.agentsReady = await page.locator(".agent b.ready").count();
  await button("Continue").click();

  if (args.mode === "full") {
    await button("Select folder").click();
    execFileSync("powershell", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", path.join(here, "pick-folder.ps1"), "-Folder", path.resolve(args.repo)], { stdio: "inherit" });
    await page.getByText("Workspace selected.").waitFor({ timeout: 60_000 });
  } else {
    await button("Try the guided example").click();
  }
  await button("Continue").click();
  await button("Enter Pytxo Desktop").click();
  step("desktop entered");
  await page.waitForTimeout(1500);

  if (args.mode === "layout") {
    await fits("work", [button("New work").last()]);
    await button("New work").last().click();
    await page.getByLabel("What should Pytxo do?").waitFor();
    await fits("new-work", [page.getByRole("button", { name: "Build plan" }).first()]);
    await page.goto(page.url().replace(/#.*$/, "#/history"));
    await page.waitForTimeout(1200);
    await fits("history");
    await page.goto(page.url().replace(/#.*$/, "#/integrations"));
    await page.getByText("Checking installed CLIs and sessions").waitFor({ state: "detached", timeout: 90_000 }).catch(() => {});
    await fits("setup-agents");
    step("layout checked");
  } else {
    const mission = readFileSync(path.resolve(args.mission), "utf8").replace(/\r/g, "").trim();
    const team = (args.team ?? "Claude Code,Cursor Agent,OpenCode,Antigravity").split(",").map((name) => name.trim()).filter(Boolean);
    await press(button("New work").last(), "New work");
    const request = page.getByLabel("What should Pytxo do?");
    await press(request, "Request");
    mark("typing");
    await request.pressSequentially(mission, { delay: 4 });
    mark("typed");
    await page.getByLabel("Agent CLI", { exact: true }).selectOption("codex");
    for (const name of team) await page.getByRole("checkbox", { name }).check();
    await press(page.getByRole("button", { name: "Build plan" }).first(), "Build plan");
    await page.getByRole("heading", { name: /Review plan|Plan blocked|Plan needs verification/ }).waitFor({ timeout: 180_000 });
    const planHeading = await page.getByRole("heading", { name: /Review plan|Plan blocked|Plan needs verification/ }).innerText();
    receipt.checks.plan = { heading: planHeading, summary: await page.locator(".plan-summary").innerText().catch(() => "") };
    mark("plan");
    await page.waitForTimeout(2500);
    await fits("plan");
    if (planHeading !== "Review plan") throw new Error(`Plan not ready: ${planHeading}`);
    step("plan ready", receipt.checks.plan);

    await press(page.getByRole("button", { name: /^Run/ }).last(), "Run");
    mark("run");
    step("run");
    const deadline = Date.now() + 20 * 60_000;
    let last = "";
    while (Date.now() < deadline) {
      await page.waitForTimeout(3000);
      const state = (await page.locator(".heading-meta").innerText().catch(() => "")).replace(/\s+/g, " ");
      if (state !== last) { step(`work: ${state}`); last = state; }
      if (!/Starting|Running/i.test(state) && /Completed|Failed|Stopped|Needs|Ready|Decision/i.test(state)) break;
    }
    mark("settled");
    await page.waitForTimeout(3000);
    receipt.checks.fleet = {
      workers: await page.locator("[data-testid=fleet-board] .worker").count(),
      passed: await page.locator('[data-testid=fleet-board] .worker[data-tone="done"]').count(),
      unchanged: await page.locator("[data-testid=fleet-board] .worker[data-unchanged]").count(),
      failed: await page.locator('[data-testid=fleet-board] .worker[data-tone="failed"]').count(),
    };
    await shot("fleet");
    step("run settled", receipt.checks.fleet);

    await press(page.getByRole("button", { name: "Review changes", exact: true }).first(), "Review changes");
    await page.locator("#run-review-title").waitFor({ timeout: 120_000 });
    await page.locator(".line-diff, .exact-diff").first().waitFor({ timeout: 60_000 });
    mark("review");
    await page.waitForTimeout(2500);
    // Read down the first file's changes, as a reviewer would.
    const diffBox = await page.locator(".line-diff, .exact-diff").first().boundingBox();
    if (diffBox) {
      const x = Math.round(diffBox.x + diffBox.width * 0.6);
      const y = Math.round(diffBox.y + Math.min(diffBox.height / 2, 240));
      receipt.pointer.push({ t: Date.now(), kind: "hover", label: "Changes", surface: "desktop", x, y });
      await page.mouse.move(x, y, { steps: 12 });
      for (let tick = 0; tick < 4; tick++) { await page.mouse.wheel(0, 260); await page.waitForTimeout(650); }
      mark("read");
    }
    const reviewed = (await page.locator(".file-row").evaluateAll((rows) => rows.map((row) => row.getAttribute("aria-label") ?? ""))).map((label) => label.replace(/^Inspect exact content for /, ""));
    receipt.checks.review = { files: reviewed, status: (await page.locator(".review-status").innerText()).replace(/\s+/g, " "), digest: (await page.locator(".package-identity code").textContent())?.trim() };
    await fits("review", [button("Apply reviewed changes")]);
    step("review", receipt.checks.review);

    // An unrelated file after review must make Apply refuse, with nothing written.
    const note = path.join(path.resolve(args.repo), "operator-note.txt");
    writeFileSync(note, "A file added after review.\n");
    mark("note");
    await press(button("Apply reviewed changes"), "Apply reviewed changes");
    await press(button("Apply exact package"), "Apply exact package");
    await page.waitForFunction(() => /stale/i.test(document.querySelector(".review-status")?.textContent ?? ""), null, { timeout: 120_000 });
    mark("stale");
    await page.waitForTimeout(3000);
    await shot("stale");
    receipt.checks.stale = { status: (await page.locator(".review-status").innerText()).replace(/\s+/g, " ") };
    step("stale refused");
    unlinkSync(note);

    await press(button("Refresh review"), "Refresh review");
    await page.waitForFunction(() => /Ready to Apply/i.test(document.querySelector(".review-status")?.textContent ?? ""), null, { timeout: 600_000 });
    receipt.checks.refreshedDigest = (await page.locator(".package-identity code").textContent())?.trim();
    mark("refreshed");
    await page.waitForTimeout(1500);
    await press(button("Apply reviewed changes"), "Apply reviewed changes");
    await press(button("Apply exact package"), "Apply exact package");
    await page.waitForFunction(() => /Applied|failed|Recovery/i.test(document.querySelector(".review-status")?.textContent ?? ""), null, { timeout: 300_000 });
    mark("applied");
    await page.waitForTimeout(3000);
    receipt.checks.apply = { status: (await page.locator(".review-status").innerText()).replace(/\s+/g, " ") };
    await shot("applied");
    step("apply", receipt.checks.apply);
    if (!/Applied/i.test(receipt.checks.apply.status)) throw new Error(`Apply did not succeed: ${receipt.checks.apply.status}`);

    await page.goto(page.url().replace(/#.*$/, "#/history"));
    await page.waitForTimeout(2000);
    await shot("history");
    receipt.frames = await stopRecording();
    receipt.checks.result = await resultApp(path.resolve(args.repo));
    step("result app", receipt.checks.result);
  }
  receipt.result = "passed";
} catch (error) {
  receipt.result = "failed";
  receipt.error = String(error?.stack ?? error);
  await shot("failure").catch(() => {});
  writeFileSync(path.join(out, "failure-page.txt"), await page.locator("body").innerText().catch(() => ""));
  throw error;
} finally {
  receipt.frames ??= await stopRecording();
  receipt.finished = new Date().toISOString();
  save();
  await browser.close().catch(() => {});
}

// The applied project, used as its own user would: served locally and opened in
// Edge (preinstalled on Windows), with the system dark preference, then switched
// to Spanish and given a task. Recorded like Desktop for the film.
async function resultApp(root) {
  const types = { ".html": "text/html", ".js": "text/javascript", ".mjs": "text/javascript", ".css": "text/css", ".json": "application/json" };
  const server = http.createServer((request, response) => {
    const file = path.join(root, decodeURIComponent(new URL(request.url, "http://localhost").pathname));
    if (!file.startsWith(root) || !existsSync(file) || !statSync(file).isFile()) { response.writeHead(404).end(); return; }
    response.writeHead(200, { "content-type": types[path.extname(file)] ?? "application/octet-stream" }).end(readFileSync(file));
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const edge = await chromium.launch({ channel: "msedge" });
  try {
    const app = await edge.newPage({ viewport: { width: 1600, height: 1000 }, colorScheme: "dark" });
    const stop = await record(app, "result-frames");
    await app.goto(`http://127.0.0.1:${server.address().port}/index.html`);
    await app.locator("#tasks li").first().waitFor();
    mark("result");
    await app.waitForTimeout(1500);
    await press(app.locator("#language-switch"), "Language", app);
    await app.locator("#language-switch").selectOption("es");
    await app.waitForFunction(() => document.documentElement.lang === "es");
    await app.waitForTimeout(1200);
    const input = app.locator("#new-task input");
    await press(input, "New task", app);
    await input.pressSequentially("Publicar la versión beta", { delay: 45 });
    await press(app.locator("#new-task button"), "Add", app);
    await app.waitForTimeout(800);
    await press(app.locator("#tasks input[type=checkbox]").last(), "Done", app);
    await app.waitForTimeout(1500);
    await app.screenshot({ path: path.join(out, "result.png") });
    const report = { lang: await app.evaluate(() => document.documentElement.lang), tasks: await app.locator("#tasks li").allInnerTexts() };
    report.frames = await stop();
    return report;
  } finally {
    await edge.close();
    server.close();
  }
}
