// Drives the fleet demo in a launched Pytxo Desktop (see launch.ps1) while
// recording the app window, and keeps every click as telemetry for the film.
//
//   node docs/demo/fleet/capture/capture.mjs --root <evidence root> --repo <fixture repo> [--until plan|run|apply] [--team "Claude Code,Cursor Agent"]
//
// The app's own rendered frames are recorded through CDP screencast (GDI
// window capture cannot see WebView2's GPU-composited content). There is no OS
// pointer: input arrives over CDP, so clicks.json keeps each click's target box
// and time for an edit to draw a pointer that matches what actually happened.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync, unlinkSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const here = import.meta.dirname;
const require = createRequire(path.join(here, "../../../../apps/desktop/package.json"));
const { chromium } = require("@playwright/test");

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
if (!args.root || !args.repo) throw new Error("usage: --root <evidence root> --repo <fixture repo> [--until plan|run|apply] [--team <names>]");
const until = args.until ?? "apply";
const launch = JSON.parse(readFileSync(path.join(args.root, "launch.json"), "utf8").replace(/^﻿/, ""));
const mission = readFileSync(path.join(here, "..", "mission.txt"), "utf8").replace(/\r/g, "").trim();
// Agents added beside Codex; only agents Setup reports ready can be ticked.
const TEAM = (args.team ?? "Claude Code,Cursor Agent,OpenCode,Antigravity").split(",").map((name) => name.trim()).filter(Boolean);

const startedAt = Date.now();
const clicks = [];
const marks = [];
const mark = (label) => { marks.push({ t: Date.now() - startedAt, label }); console.log(`${((Date.now() - startedAt) / 1000).toFixed(1)}s ${label}`); };

const browser = await chromium.connectOverCDP(`http://127.0.0.1:${launch.cdp_port}`);
const page = browser.contexts().flatMap((context) => context.pages())[0];
const framesDir = path.join(args.root, "frames");
mkdirSync(framesDir, { recursive: true });
const frames = [];
const cdp = await page.context().newCDPSession(page);
cdp.on("Page.screencastFrame", ({ data, metadata, sessionId }) => {
  const file = path.join(framesDir, `${String(frames.length).padStart(6, "0")}.jpg`);
  writeFileSync(file, Buffer.from(data, "base64"));
  frames.push({ file, t: metadata.timestamp * 1000 });
  cdp.send("Page.screencastFrameAck", { sessionId }).catch(() => {});
});
await cdp.send("Page.startScreencast", { format: "jpeg", quality: 92, everyNthFrame: 1 });
const click = async (locator, label) => {
  await locator.scrollIntoViewIfNeeded();
  const box = await locator.boundingBox();
  await page.waitForTimeout(450);
  clicks.push({ t: Date.now() - startedAt, label, box, viewport: page.viewportSize() ?? (await page.evaluate(() => ({ width: innerWidth, height: innerHeight }))) });
  await locator.click();
};
const status = async () => (await page.locator(".review-status").innerText().catch(() => "")).replace(/\s+/g, " ");
const shot = (name) => page.screenshot({ path: path.join(args.root, `${name}.png`) });

try {
  mark("onboarding");
  await click(page.getByRole("button", { name: "Get started" }), "Get started");
  await page.getByText("Finding your coding agents").waitFor({ state: "detached", timeout: 60_000 }).catch(() => {});
  await page.waitForTimeout(1500);
  await shot("agents");
  await click(page.getByRole("button", { name: "Continue", exact: true }), "Continue");
  await click(page.getByRole("button", { name: "Select folder" }), "Select folder");
  execFileSync("powershell", ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", path.join(here, "pick-folder.ps1"), "-Folder", path.resolve(args.repo)], { stdio: "inherit" });
  await page.getByText("Workspace selected.").waitFor({ timeout: 30_000 });
  await click(page.getByRole("button", { name: "Continue", exact: true }), "Continue");
  await click(page.getByRole("button", { name: "Enter Pytxo Desktop" }), "Enter Pytxo Desktop");

  mark("compose");
  await page.waitForTimeout(1500);
  await click(page.getByRole("button", { name: "New work" }).last(), "New work");
  const request = page.getByLabel("What should Pytxo do?");
  await click(request, "Request");
  await request.pressSequentially(mission, { delay: 6 });
  await page.getByLabel("Agent CLI", { exact: true }).selectOption("codex");
  for (const name of TEAM) await click(page.getByRole("checkbox", { name }), name);
  await click(page.getByRole("button", { name: "Build plan" }).first(), "Build plan");
  await page.getByRole("heading", { name: /Review plan|Plan blocked/ }).waitFor({ timeout: 180_000 });
  await page.waitForTimeout(2500);
  await shot("plan");
  mark("plan ready");
  if (until === "plan") throw "stop";

  await click(page.getByRole("button", { name: /^Run/ }).last(), "Run");
  mark("run");
  const deadline = Date.now() + 45 * 60_000;
  let last = "";
  while (Date.now() < deadline) {
    await page.waitForTimeout(5000);
    const state = await page.locator(".heading-meta").innerText().catch(() => "");
    if (state !== last) { mark(`work: ${state.replace(/\s+/g, " ")}`); last = state; }
    if (!/Starting|Running/i.test(state) && /Completed|Failed|Stopped|Needs|Ready/i.test(state)) break;
  }
  await page.waitForTimeout(4000);
  await shot("board");
  if (until === "run") throw "stop";

  mark("review");
  await click(page.getByRole("button", { name: "Review changes" }).first(), "Review changes");
  await page.locator("#run-review-title").waitFor({ timeout: 60_000 });
  await page.waitForTimeout(3000);
  await shot("review");
  const preparedDigest = (await page.locator(".package-identity code").textContent().catch(() => ""))?.trim();

  mark("stale");
  const note = path.join(args.repo, "operator-note.txt");
  writeFileSync(note, "A file added after review.\n");
  await click(page.getByRole("button", { name: "Apply reviewed changes" }), "Apply reviewed changes");
  await click(page.getByRole("button", { name: "Apply exact package" }), "Apply exact package");
  await page.waitForFunction(() => /stale/i.test(document.querySelector(".review-status")?.textContent ?? ""), null, { timeout: 60_000 });
  await page.waitForTimeout(3000);
  await shot("stale");
  unlinkSync(note);

  mark("refresh");
  await click(page.getByRole("button", { name: "Refresh review" }), "Refresh review");
  const refreshDeadline = Date.now() + 10 * 60_000;
  while (Date.now() < refreshDeadline && !/Ready to Apply/i.test(await status())) await page.waitForTimeout(3000);
  await click(page.getByRole("button", { name: "Apply reviewed changes" }), "Apply reviewed changes");
  await page.waitForTimeout(1200);
  await click(page.getByRole("button", { name: "Apply exact package" }), "Apply exact package");
  while (!/Applied|failed|Recovery/i.test(await status())) await page.waitForTimeout(2000);
  await page.waitForTimeout(3000);
  await shot("applied");
  mark(`apply: ${await status()}`);
  writeFileSync(path.join(args.root, "stale-digest.txt"), `${preparedDigest ?? ""}\n`);
} catch (error) {
  if (error !== "stop") throw error;
} finally {
  await cdp.send("Page.stopScreencast").catch(() => {});
  await browser.close().catch(() => {});
  // Screencast frames arrive when the page changes; hold each until the next.
  const entry = (frame) => `file '${frame.file.split(path.sep).join("/")}'`;
  const list = frames.map((frame, index) => `${entry(frame)}\nduration ${(((frames[index + 1]?.t ?? frame.t + 1000) - frame.t) / 1000).toFixed(3)}`);
  writeFileSync(path.join(args.root, "frames.txt"), ["ffconcat version 1.0", ...list, entry(frames.at(-1)), ""].join("\n"));
  execFileSync("ffmpeg", ["-hide_banner", "-loglevel", "error", "-f", "concat", "-safe", "0", "-i", path.join(args.root, "frames.txt"), "-vf", "fps=30,scale=trunc(iw/2)*2:trunc(ih/2)*2,format=yuv420p", "-c:v", "libx264", "-crf", "16", "-y", path.join(args.root, "capture.mp4")], { stdio: "inherit" });
  // Video time 0 is the first frame; click and mark times are relative to startedAt.
  writeFileSync(path.join(args.root, "clicks.json"), JSON.stringify({ startedAt: new Date(startedAt).toISOString(), videoStartsAtMs: frames[0] ? frames[0].t - startedAt : null, frames: frames.length, clicks, marks }, null, 2));
  console.log(`capture: ${path.join(args.root, "capture.mp4")} from ${frames.length} frames`);
}
