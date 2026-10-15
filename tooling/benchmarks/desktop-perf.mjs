// Measures Pytxo Desktop speed on a native release exe against a seeded history
// (crates/pytxo-store/examples/seed_history.rs): process launch to ready shell,
// per-screen open time with its IPC data loads, History scroll frame times,
// time to show the newest line of a long recorded worker output.
//
//   node tooling/benchmarks/desktop-perf.mjs --exe target/release/pytxo-desktop.exe \
//     --fixture D:/fixture --pristine D:/fixture-pristine --out D:/perf/baseline [--runs 5] [--port 9361]
//
// Each run restores the fixture from the pristine copy (domain ids are absolute
// paths, so the fixture path never changes) and reuses one warmed WebView2
// profile, so numbers reflect a returning user rather than first install.
// Input arrives over CDP (WebView2 --remote-debugging-port); no OS pointer.
import { spawn, execFileSync } from "node:child_process";
import { cpSync, mkdirSync, rmSync, writeFileSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const here = import.meta.dirname;
const require = createRequire(path.join(here, "../../apps/desktop/package.json"));
const { chromium } = require("@playwright/test");

const args = Object.fromEntries(process.argv.slice(2).reduce((pairs, value, index, all) => (value.startsWith("--") ? [...pairs, [value.slice(2), all[index + 1]]] : pairs), []));
for (const key of ["exe", "fixture", "pristine", "out"]) if (!args[key]) throw new Error(`--${key} is required`);
const runs = Number(args.runs ?? 5);
const port = Number(args.port ?? 9361);
const exe = path.resolve(args.exe);
const out = path.resolve(args.out);
const webview = path.join(out, "webview");
mkdirSync(out, { recursive: true });
const log = (...parts) => console.error(new Date().toISOString().slice(11, 19), ...parts);
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

// In-page helpers: invoke timing, readiness waits, frame sampling.
// In-page helpers. Tauri's internals are frozen, but its IPC goes through
// window.fetch("http://ipc.localhost/<command>"), which can be observed.
const PAGE_HELPERS = () => {
  if (window.__perf) return;
  performance.setResourceTimingBufferSize(100000);
  const calls = [];
  let inflight = 0;
  const original = window.fetch;
  window.fetch = function (input) {
    const url = String(input?.url ?? input);
    if (!url.includes("ipc.localhost")) return original.apply(this, arguments);
    const call = { cmd: new URL(url).pathname.slice(1), start: performance.now(), end: null };
    calls.push(call);
    inflight += 1;
    return original.apply(this, arguments).finally(() => { call.end = performance.now(); inflight -= 1; });
  };
  const frame = () => new Promise((resolve) => requestAnimationFrame(() => resolve(performance.now())));
  // Visible: first frame where ready() holds. Settled: ready() and no IPC in
  // flight for six consecutive frames (~100 ms), so chained loads count.
  const measure = async (trigger, ready, timeout = 60000) => {
    const from = calls.length;
    const t0 = performance.now();
    trigger();
    let visible = null;
    let quiet = 0;
    while (performance.now() - t0 < timeout) {
      const now = await frame();
      const ok = ready();
      if (ok && visible === null) visible = now;
      quiet = ok && inflight === 0 ? quiet + 1 : 0;
      if (quiet >= 6) {
        const ipc = calls.slice(from).filter((call) => call.end !== null && call.end <= now);
        return { visible_ms: visible - t0, settled_ms: now - t0, ipc: ipc.map((call) => ({ cmd: call.cmd, ms: Math.round(call.end - call.start) })) };
      }
    }
    return { timeout: true, visible_ms: visible === null ? null : visible - t0, pending: calls.slice(from).filter((call) => call.end === null).map((call) => call.cmd) };
  };
  window.__perf = { calls, measure, frame, inflight: () => inflight };
};

// seed_history.rs defaults: 400 runs, 5000 events per worker in the newest.
const OUTPUT_RUN = "Run 399 in workspace 1";
const OUTPUT_MARKER = "04999 ";

const SCREENS = {
  history: { nav: "History", ready: () => !!document.querySelector(".screen.history .rows .row") && !document.querySelector(".screen.history p[role=status]") },
  setup: { nav: "Setup", ready: () => !!document.querySelector(".setup-content") && !document.querySelector(".loading-state") },
  work: { nav: "Work", ready: () => !!document.querySelector(".work-content") && !document.querySelector(".loading-state") },
};

async function openScreen(page, name) {
  const screen = SCREENS[name];
  return page.evaluate(({ nav, ready }) => {
    const button = document.querySelector(`nav[aria-label="Primary"] [aria-label="${nav}"], nav[aria-label="System"] [aria-label="${nav}"]`);
    if (!button) throw new Error(`no nav button ${nav}`);
    return window.__perf.measure(() => button.click(), new Function(`return (${ready})()`));
  }, { nav: screen.nav, ready: screen.ready.toString() });
}

// History detail for an applied run, then its Review screen.
async function openReview(page) {
  const detail = await page.evaluate(() => {
    const row = [...document.querySelectorAll(".screen.history .rows .row")].find((r) => /Apply recorded/.test(r.textContent) && !r.classList.contains("selected"));
    if (!row) return null;
    const ready = () => { const action = document.querySelector(".outcome-inspector .outcome-action"); return !!action && !action.disabled && /View applied result/.test(action.textContent); };
    return window.__perf.measure(() => row.click(), ready);
  });
  const review = await page.evaluate(() => {
    const action = document.querySelector(".outcome-inspector .outcome-action");
    if (!action) return null;
    return window.__perf.measure(() => action.click(), () => !!document.querySelector(".review-screen .review-header h1"));
  });
  return { history_detail: detail, review };
}

// Recorded output of a worker in the newest seeded run (--tail-events lines,
// last one "04999 ..."): click Open output until the newest line is on screen.
async function openOutput(page) {
  await page.locator('nav[aria-label="Primary"] [aria-label="Work"]').click();
  await page.locator(".workspace-switcher button.switcher").click();
  await page.getByRole("option", { name: /^workspace-01/ }).click();
  await page.locator("aside").getByText(OUTPUT_RUN, { exact: true }).first().click();
  await page.getByRole("button", { name: "List", exact: true }).click();
  await page.locator(".list-view button.row").first().click();
  await page.getByRole("button", { name: "Open output", exact: true }).waitFor();
  return page.evaluate((marker) => {
    const action = [...document.querySelectorAll("section.inspection button")].find((button) => button.textContent.trim() === "Open output");
    const ready = () => !!document.querySelector('[aria-label="Recorded agent output"]')?.textContent.includes(marker);
    return window.__perf.measure(() => action.click(), ready, 120000);
  }, OUTPUT_MARKER);
}

async function scrollHistory(page) {
  return page.evaluate(async () => {
    const row = document.querySelector(".screen.history .rows .row");
    let scroller = row?.parentElement;
    while (scroller && scroller.scrollHeight <= scroller.clientHeight + 1) scroller = scroller.parentElement;
    if (!scroller) return null;
    scroller.scrollTop = 0;
    await window.__perf.frame();
    const deltas = [];
    let last = await window.__perf.frame();
    const max = scroller.scrollHeight - scroller.clientHeight;
    for (let i = 0; i < 120 && scroller.scrollTop < max - 1; i += 1) {
      scroller.scrollTop += 60;
      const now = await window.__perf.frame();
      deltas.push(now - last);
      last = now;
    }
    deltas.sort((a, b) => a - b);
    const q = (p) => deltas[Math.min(deltas.length - 1, Math.floor(p * deltas.length))];
    return { frames: deltas.length, rows: document.querySelectorAll(".screen.history .rows .row").length, p50_ms: q(0.5), p95_ms: q(0.95), max_ms: deltas.at(-1), over_33ms: deltas.filter((d) => d > 33.4).length };
  });
}

function processMemoryMb(pid) {
  // Host plus its WebView2 children, private working set.
  const script = `$ids=@(${pid}); $all=Get-CimInstance Win32_Process; do { $n=$ids.Count; $ids=@($ids + ($all | Where-Object { $ids -contains $_.ParentProcessId } | ForEach-Object ProcessId) | Select-Object -Unique) } while ($ids.Count -gt $n); (Get-Process -Id $ids -ErrorAction SilentlyContinue | Measure-Object -Property PrivateMemorySize64 -Sum).Sum`;
  return Math.round(Number(execFileSync("powershell", ["-NoProfile", "-Command", script]).toString().trim()) / 1048576);
}

async function waitForCdp() {
  for (let i = 0; i < 2400; i += 1) {
    try { const res = await fetch(`http://127.0.0.1:${port}/json/version`); if (res.ok) return; } catch { /* not up */ }
    await sleep(25);
  }
  throw new Error("CDP never came up");
}

async function launch() {
  // Single-instance Desktop forwards a second launch to a survivor; never measure that.
  try { execFileSync("powershell", ["-NoProfile", "-Command", `Get-CimInstance Win32_Process | Where-Object { $_.ExecutablePath -eq '${exe}' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }`], { stdio: "ignore" }); } catch { /* none */ }
  rmSync(args.fixture, { recursive: true, force: true });
  cpSync(args.pristine, args.fixture, { recursive: true });
  const home = path.join(args.fixture, "home");
  const launchedAt = Date.now();
  const child = spawn(exe, [], {
    cwd: home,
    env: {
      ...process.env,
      PYTXO_HOME: home,
      PYTXO_TRUST_STORE: path.join(home, "trusted-domains.json"),
      WEBVIEW2_USER_DATA_FOLDER: webview,
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    },
    stdio: "ignore",
  });
  log("launched", child.pid);
  await waitForCdp();
  log("cdp up");
  const browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
  let page;
  for (let i = 0; i < 400 && !page; i += 1) {
    page = browser.contexts().flatMap((context) => context.pages()).find((p) => !p.url().startsWith("about:"));
    if (!page) await sleep(25);
  }
  return { child, browser, page, launchedAt };
}

async function stop({ child, browser }) {
  await browser.close().catch(() => {});
  try { execFileSync("taskkill", ["/PID", String(child.pid), "/T", "/F"], { stdio: "ignore" }); } catch { /* gone */ }
  await sleep(1500);
}

async function measureRun(index) {
  const session = await launch();
  try { return await measureSession(session, index); } finally { await stop(session); }
}

async function measureSession(session, index) {
  const { page, launchedAt } = session;
  await page.waitForFunction(() => performance.getEntriesByName("pytxo:ready").length > 0, null, { timeout: 120000 });
  const startup = await page.evaluate(() => {
    const ready = performance.getEntriesByName("pytxo:ready")[0]?.startTime ?? null;
    const fcp = performance.getEntriesByName("first-contentful-paint")[0]?.startTime ?? null;
    const ipc = performance.getEntriesByType("resource").filter((entry) => entry.name.includes("ipc.localhost"));
    const byCmd = {};
    for (const entry of ipc) {
      if (ready !== null && entry.startTime > ready) continue;
      const cmd = new URL(entry.name).pathname.slice(1);
      byCmd[cmd] ??= { count: 0, ms: 0 };
      byCmd[cmd].count += 1;
      byCmd[cmd].ms += entry.duration;
    }
    return { time_origin: performance.timeOrigin, fcp_ms: fcp, ready_ms: ready, ipc_before_ready: byCmd, heap_mb: Math.round((performance.memory?.usedJSHeapSize ?? 0) / 1048576) };
  });
  const result = {
    run: index,
    launch_to_navigation_ms: Math.round(startup.time_origin - launchedAt),
    launch_to_fcp_ms: startup.fcp_ms === null ? null : Math.round(startup.time_origin - launchedAt + startup.fcp_ms),
    launch_to_ready_ms: startup.ready_ms === null ? null : Math.round(startup.time_origin - launchedAt + startup.ready_ms),
    ipc_before_ready: startup.ipc_before_ready,
    heap_after_ready_mb: startup.heap_mb,
  };
  await page.evaluate(PAGE_HELPERS);
  await sleep(1500);
  result.memory_after_ready_mb = processMemoryMb(session.child.pid);
  result.screens = [];
  for (const name of ["history", "setup", "work", "history", "work", "setup", "history"]) {
    log("open", name);
    result.screens.push({ screen: name, ...(await openScreen(page, name)) });
    await sleep(400);
  }
  log("scroll");
  result.history_scroll = await scrollHistory(page);
  log("review");
  Object.assign(result, await openReview(page));
  await page.screenshot({ path: path.join(out, `run-${index}-review.png`) }).catch(() => {});
  // The launch-time resource buffer holds only 250 entries; a webview reload
  // with a larger buffer shows every IPC call the frontend makes before ready.
  log("reload");
  await page.addInitScript(() => performance.setResourceTimingBufferSize(100000));
  await page.evaluate(() => location.reload());
  await page.waitForFunction(() => performance.getEntriesByName("pytxo:ready").length > 0, null, { timeout: 120000 });
  result.reload = await page.evaluate(() => {
    performance.setResourceTimingBufferSize(100000);
    const ready = performance.getEntriesByName("pytxo:ready")[0].startTime;
    const byCmd = {};
    let calls = 0;
    for (const entry of performance.getEntriesByType("resource")) {
      if (!entry.name.includes("ipc.localhost") || entry.startTime > ready) continue;
      const cmd = new URL(entry.name).pathname.slice(1);
      byCmd[cmd] ??= { count: 0, ms: 0 };
      byCmd[cmd].count += 1;
      byCmd[cmd].ms = Math.round(byCmd[cmd].ms + entry.duration);
      calls += 1;
    }
    return { ready_ms: Math.round(ready), ipc_calls: calls, ipc: byCmd };
  });
  log("output");
  await page.evaluate(PAGE_HELPERS);
  result.output = await openOutput(page);
  return result;
}

if (!existsSync(webview)) {
  // Warm-up launch: mark onboarding done in this profile so later runs land in the shell.
  const session = await launch();
  await session.page.waitForLoadState("domcontentloaded");
  await session.page.evaluate(() => {
    localStorage.setItem("pytxo-desktop-onboarding-version", "1.1.0");
    localStorage.setItem("pytxo-deck-setup-v1", "complete");
  });
  // A hard kill can drop unflushed localStorage: reload into the shell and let it settle.
  await session.page.reload();
  await session.page.waitForFunction(() => performance.getEntriesByName("pytxo:ready").length > 0, null, { timeout: 120000 });
  await sleep(10000);
  await stop(session);
}
const results = [];
for (let index = 0; index < runs; index += 1) {
  const result = await measureRun(index);
  console.log(JSON.stringify({ run: index, ready: result.launch_to_ready_ms, screens: result.screens.map((s) => `${s.screen}:${Math.round(s.settled_ms)}`).join(" "), detail: result.history_detail && Math.round(result.history_detail.settled_ms), review: result.review && Math.round(result.review.settled_ms), scroll_p95: result.history_scroll?.p95_ms, reload_ready: result.reload?.ready_ms, reload_ipc: result.reload?.ipc_calls, output: result.output && Math.round(result.output.visible_ms) }));
  results.push(result);
}
writeFileSync(path.join(out, "results.json"), JSON.stringify({ exe, runs: results }, null, 2));
