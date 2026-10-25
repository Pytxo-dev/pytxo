import { expect, test, type Page } from "@playwright/test";
import { fleetLinks, fleetWall, homeTilde, landedTile, runKey, runVerdict, wallLayout } from "../src/lib/fleet-wall";
import type { AgentDto, CatalogEntryStatus, FleetNodeDto, RunDto, RunReviewDto } from "../src/lib/types";
import { completeOnboarding } from "./helpers";

const run = (domain_id: string, id: string, status: string, started_at: string) =>
  ({ id, domain_id, repo_root: domain_id, status, started_at }) as unknown as RunDto;
const agent = (domain_id: string, run_id: string, id: string, task_id: string, status: string, exit_code: number | null = null, wave = 0) =>
  ({ id, domain_id, run_id, task_id, wave, status, exit_code, root_id: null, launcher: null }) as AgentDto;
const domain = (domain_id: string, repo_root: string) => ({ domain_id, repo_root }) as CatalogEntryStatus;
/** A prepared package: `files` changed paths, one combined check that `passed`. */
const manifest = (files = 1, passed = true) => ({ version: 3, files: Array(files).fill({}), candidate_verification: { version: 1, checks: [{ command: "npm test", passed }] } });
const review = (apply_status: string, prepared_manifest: unknown = null, last_apply_error: unknown = null) =>
  ({ apply_status, recovery_state: null, last_apply_error, prepared_manifest }) as unknown as RunReviewDto;
const failure = (code: string, message: string) => ({ at: "2026-10-23T10:00:00Z", code, message, attempt_id: null, rollback_confirmed: false });

test.describe("wall model", () => {
  const domains = [domain("d1", "C:\\work\\api"), domain("d2", "/home/me/web")];
  const runs = [
    run("d2", "run-1", "running", "2026-10-23T10:00:05Z"),
    run("d1", "run-1", "running", "2026-10-23T10:00:00Z"),
    run("d1", "run-old", "completed", "2026-10-23T09:00:00Z"),
  ];
  const agents = [
    agent("d1", "run-1", "a10", "task-10", "running"),
    agent("d1", "run-1", "a2", "task-2", "completed", 0),
    agent("d1", "run-1", "a3", "task-3", "failed", 1),
    agent("d2", "run-1", "b1", "task-1", "queued"),
    agent("d2", "run-1", "b2", "task-2", "completed", 2),
    agent("d2", "run-1", "b3", "task-3", "stopped"),
    agent("d1", "run-old", "old", "task-1", "completed", 0),
  ];

  test("groups active runs by workspace, keeps same run ids apart, and counts only listed workers", () => {
    const wall = fleetWall({ domains, runs, agents });
    expect(wall.groups.map((group) => group.repo)).toEqual(["api", "web"]);
    expect(wall.runs.map(runKey)).toEqual(["d1\nrun-1", "d2\nrun-1"]);
    expect(wall.groups[0].tiles.map((tile) => [tile.agent.task_id, tile.label])).toEqual([["task-2", "done"], ["task-3", "failed"], ["task-10", "working"]]);
    expect(wall.groups[1].tiles.map((tile) => tile.label)).toEqual(["queued", "exit 2", "stopped"]);
    expect(wall.counts).toEqual({ working: 1, done: 1, failed: 2, queued: 1, other: 1, agents: 6 });
    expect(wall.groups[1].counts).toMatchObject({ failed: 1, queued: 1, other: 1, agents: 3 });
  });

  test("a run that finishes stays on the wall only while it is kept", () => {
    const first = fleetWall({ domains, runs, agents });
    const finished = runs.map((entry) => entry.domain_id === "d2" ? { ...entry, status: "completed" } : entry);
    expect(fleetWall({ domains, runs: finished, agents }, first.keys).runs.map(runKey)).toEqual(["d1\nrun-1", "d2\nrun-1"]);
    expect(fleetWall({ domains, runs: finished, agents }).runs.map(runKey)).toEqual(["d1\nrun-1"]);
    expect(fleetWall({ domains, runs: finished, agents }).counts.agents).toBe(3);
  });

  test("done is the agent's claim until its run's combined change is verified", () => {
    const [done, failed] = ["task-2", "task-3"].map((task) => fleetWall({ domains, runs, agents }).groups[0].tiles.find((tile) => tile.agent.task_id === task)!);
    expect(done).toMatchObject({ tone: "claimed", label: "done" });
    expect(landedTile(done, new Set())).toBe(done);
    expect(landedTile(done, new Set([runKey(done.run)]))).toMatchObject({ tone: "verified", label: "done" });
    expect(landedTile(failed, new Set([runKey(failed.run)]))).toBe(failed);
  });

  test("a run's verdict is Review's own, and only a verified combined change lands", () => {
    const settled = (apply_status: string, extra: Partial<RunDto> = {}) => ({ ...run("d1", "r", "completed", "2026-10-23T10:00:00Z"), apply_status, ...extra }) as RunDto;
    expect(runVerdict(run("d1", "r", "running", "2026-10-23T10:00:00Z"), 3, null)).toEqual({ tone: "active", label: "3 working", landed: false });
    expect(runVerdict(settled("preparing"), 0, null)).toEqual({ tone: "active", label: "Checking the combined change…", landed: false });
    // Only the prepared package shows verification, so a ready run claims nothing until it is read.
    expect(runVerdict(settled("ready"), 0, null)).toBeNull();
    expect(runVerdict(settled("ready"), 0, review("ready", manifest()))).toEqual({ tone: "verified", label: "Combined change verified · Ready to Apply", landed: true });
    expect(runVerdict(settled("ready"), 0, review("ready", manifest(1, false)))).toEqual({ tone: "unknown", label: "Checks needed before Apply", landed: false });
    expect(runVerdict(settled("ready"), 0, review("ready", { ...manifest(), version: 2 }))).toMatchObject({ tone: "unknown", landed: false });
    expect(runVerdict(settled("ready"), 0, review("ready", manifest(0)))).toEqual({ tone: "unknown", label: "Nothing to Apply", landed: false });
    const checkFailed = failure("candidate_check_failed", "task-1: npm run test -- --ci failed on the combined change");
    expect(runVerdict(settled("review_failed"), 0, review("review_failed", null, checkFailed))).toEqual({ tone: "refuted", label: "Combined check failed: npm run test -- --ci", landed: false });
    expect(runVerdict(settled("review_failed", { last_apply_error: checkFailed }), 0, null)).toMatchObject({ label: "Combined check failed: npm run test -- --ci" });
    expect(runVerdict(settled("review_failed"), 0, review("review_failed", null, failure("review_preparation_failed", "disk full")))).toEqual({ tone: "refuted", label: "Could not prepare the review", landed: false });
    expect(runVerdict(settled("stale"), 0, review("stale", manifest()))).toEqual({ tone: "unknown", label: "Review is stale", landed: false });
    expect(runVerdict(settled("applied"), 0, review("applied", manifest()))).toEqual({ tone: "verified", label: "Applied successfully", landed: true });
  });

  test("shows the home folder as ~ only where whole folder names match", () => {
    expect(homeTilde("wrote C:\\Users\\Matt\\fw\\a.ts", "C:\\Users\\Matt")).toBe("wrote ~\\fw\\a.ts");
    expect(homeTilde("wrote c:/users/matt/fw/a.ts", "C:\\Users\\Matt")).toBe("wrote ~/fw/a.ts");
    expect(homeTilde("C:\\Users\\Matthew\\x", "C:\\Users\\Matt")).toBe("C:\\Users\\Matthew\\x");
    expect(homeTilde("cd /home/me/web && ls /home/me", "/home/me/")).toBe("cd ~/web && ls ~");
    expect(homeTilde("/var/home/me/x", "/home/me")).toBe("/var/home/me/x");
    expect(homeTilde("C:\\Users\\Matt", null)).toBe("C:\\Users\\Matt");
  });

  test("links a fleet only when one of its nodes dispatched a run on the wall", () => {
    const node = (domain_id: string, domain_run_id: string | null) => ({ node_id: "n", domain_id, domain_run_id, wave: 0, status: "running" }) as FleetNodeDto;
    const wall = fleetWall({ domains, runs, agents });
    expect(fleetLinks(wall.runs, { linked: [node("d2", "run-1")], other: [node("d2", "run-old"), node("d1", null)] })).toEqual(["linked"]);
  });

  test("fits a 48-agent fleet as eight columns by six rows at 1080p", () => {
    const layout = wallLayout(Array(6).fill(8), 1708, 858);
    expect(layout).toMatchObject({ blocks: 2, columns: 4, fits: true });
    expect(layout.tileWidth).toBeGreaterThanOrEqual(190);
    expect(layout.tileHeight).toBeGreaterThanOrEqual(110);
  });

  test("keeps 104 agents on one screen and reports when nothing fits", () => {
    const layout = wallLayout([...Array(13).fill(8)], 1708, 858);
    expect(layout.fits).toBe(true);
    expect(layout.tileWidth).toBeGreaterThanOrEqual(120);
    expect(layout.tileHeight).toBeGreaterThanOrEqual(56);
    expect(wallLayout(Array(40).fill(8), 900, 500).fits).toBe(false);
  });

  test("a narrow window scrolls down instead of overflowing sideways", () => {
    const layout = wallLayout([8, 8, 4, 3], 317, 600);
    const used = layout.blocks * (layout.columns * layout.tileWidth + (layout.columns - 1) * 8 + 16) + (layout.blocks - 1) * 12;
    expect(layout.fits).toBe(false);
    expect(used).toBeLessThanOrEqual(317);
    expect(layout.tileHeight).toBe(92);
  });
});

test("the wall shows every workspace's agents with counts and run ids from the snapshot", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-wall-v1": "1", "pytxo-preview-observe-polls-v1": "1" });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Wall", exact: true }).click();

  const wall = page.getByTestId("fleet-wall");
  await expect(wall).toBeVisible();
  await expect(wall.getByRole("heading", { name: "All agents" })).toBeFocused();
  await expect(wall.getByRole("button", { name: "Wall", exact: true })).toHaveAttribute("aria-pressed", "true");
  await expect(wall.locator(".group .tui-legend b")).toHaveText(["fw1", "fw2", "fw3", "pytxo"]);
  for (const [repo, count] of [["fw1", 8], ["fw2", 8], ["fw3", 4], ["pytxo", 3]] as const) {
    await expect(wall.getByRole("region", { name: `${repo}: ${count} agents` }).locator("button.tile")).toHaveCount(count);
  }

  const stat = (kind: string) => wall.locator(`.stats [data-kind="${kind}"] dd`);
  for (const [kind, value] of [["working", "16"], ["done", "5"], ["failed", "1"], ["queued", "1"], ["runs", "4"], ["repos", "4"]]) await expect(stat(kind)).toHaveText(value);
  await expect(stat("other")).toHaveCount(0);

  const evidence = wall.getByLabel("Runs these counts come from");
  await expect(evidence).toContainText("Counts from 4 runs:");
  for (const id of ["fw1 wall-run-1", "fw2 wall-run-2", "fw3 wall-run-3", "pytxo run-8f2c", "fleet fleet-run-preview"]) await expect(evidence).toContainText(id);

  const working = wall.getByRole("button", { name: /^fw1 task-1: working, \d+:\d\d\. Claude Code\. Open run\.$/ });
  await expect(working).toContainText("Browser fixture: task-1 writing its change");
  await expect(working).toContainText("Browser fixture: task-1 running its checks");
  await expect(working).not.toContainText("reading the workspace");
  await expect(wall.getByRole("button", { name: /^fw2 task-6: failed/ })).toHaveAttribute("data-tone", "refuted");
  // Its run is still working, so a finished agent's exit 0 is only its claim.
  await expect(wall.getByRole("button", { name: /^fw1 task-7: done/ })).toHaveAttribute("data-tone", "claimed");

  // Every tile's output arrives in one read per poll, not one per worker.
  const reads = () => page.evaluate(() => Number(localStorage.getItem("pytxo-preview-tail-batch-reads-v1") ?? "0"));
  const before = await reads();
  await page.waitForTimeout(3_200);
  expect(await reads() - before).toBeLessThanOrEqual(4);

  await wall.getByRole("button", { name: /^fw2 task-6: failed/ }).click();
  await expect(wall).toHaveCount(0);
  await expect(page.getByRole("heading", { level: 1 })).toHaveText("Work in fw2");
  await expect(page.getByRole("button", { name: "Run", exact: true })).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("button", { name: "Wall", exact: true }).click();
  await expect(page.getByTestId("fleet-wall")).toBeVisible();
  await page.getByRole("button", { name: "Run", exact: true }).click();
  await expect(page.getByTestId("fleet-wall")).toHaveCount(0);
});

for (const [agents, minWidth, minHeight] of [[48, 185, 110], [100, 120, 56]] as const) {
  test(`${agents} agents fit one 1080p screen without scrolling`, async ({ page }) => {
    await page.setViewportSize({ width: 1920, height: 1080 });
    await completeOnboarding(page, { "pytxo-preview-wall-v1": String(agents) });
    await page.goto("/#/work");
    await page.getByRole("button", { name: "Wall", exact: true }).click();

    const wall = page.getByTestId("fleet-wall");
    await expect(wall.locator("button.tile")).toHaveCount(agents);
    await expect(wall.locator('.stats [data-kind="working"] dd')).toHaveText(String(agents));
    await expect(wall.locator('.stats [data-kind="runs"] dd')).toHaveText(String(Math.ceil(agents / 8)));
    await expect(wall.locator(".tile .out").first()).toContainText("Browser fixture");
    const body = await wall.locator(".wall-body").evaluate((element) => ({ overflow: element.scrollHeight - element.clientHeight, sideways: element.scrollWidth - element.clientWidth }));
    expect(body).toEqual({ overflow: 0, sideways: 0 });
    const tile = await wall.locator("button.tile").first().boundingBox();
    expect(tile?.width).toBeGreaterThanOrEqual(minWidth);
    expect(tile?.height).toBeGreaterThanOrEqual(minHeight);
  });
}

test.describe("landing on the wall", () => {
  test.beforeEach(async ({ page }) => {
    await page.setViewportSize({ width: 1600, height: 1000 });
    await completeOnboarding(page, { "pytxo-preview-wall-v1": "land", "pytxo-preview-observe-polls-v1": "1" });
    await page.goto("/#/work");
    await page.getByRole("button", { name: "Wall", exact: true }).click();
  });
  const pane = (page: Page, repo: string) => page.getByTestId("fleet-wall").getByRole("region", { name: new RegExp(`^${repo}: `) });

  test("done tiles stay hollow until the combined change is verified, and each run says why", async ({ page }) => {
    const wall = page.getByTestId("fleet-wall");
    const states = [
      ["working", "active", "2 working", ["active", "active"]],
      ["checking", "active", "Checking the combined change…", ["claimed", "claimed"]],
      ["ready", "verified", "Combined change verified · Ready to Apply", ["verified", "verified"]],
      ["check-failed", "refuted", "Combined check failed: npm test", ["claimed", "claimed"]],
      ["empty", "unknown", "Nothing to Apply", ["claimed", "claimed"]],
      ["stale", "unknown", "Review is stale", ["claimed", "claimed"]],
      ["failed", "refuted", "Could not prepare the review", ["claimed", "refuted"]],
      ["applied", "verified", "Applied successfully", ["verified", "verified"]],
    ] as const;
    for (const [repo, tone, label, tiles] of states) {
      const verdict = pane(page, repo).locator("button.verdict");
      await expect(verdict.locator("> span:last-child")).toHaveText(label);
      await expect(verdict).toHaveAttribute("data-tone", tone);
      await expect(pane(page, repo).locator("button.tile")).toHaveCount(2);
      for (const [index, tileTone] of tiles.entries()) await expect(pane(page, repo).locator("button.tile").nth(index)).toHaveAttribute("data-tone", tileTone);
    }
    const stat = (kind: string) => wall.locator(`.stats [data-kind="${kind}"] dd`);
    for (const [kind, value] of [["working", "2"], ["done", "13"], ["failed", "1"], ["ready", "2/8"], ["runs", "8"], ["repos", "8"]]) await expect(stat(kind)).toHaveText(value);

    // Hollow is a transparent face; verified fills it, at once under reduced motion.
    const face = (repo: string) => pane(page, repo).locator("button.tile").first().evaluate((tile) => getComputedStyle(tile).backgroundColor);
    expect(await face("check-failed")).toBe("rgba(0, 0, 0, 0)");
    expect(await face("ready")).not.toBe("rgba(0, 0, 0, 0)");
    const fill = () => pane(page, "ready").locator("button.tile").first().evaluate((tile) => Math.max(...getComputedStyle(tile).transitionDuration.split(",").map(parseFloat)));
    expect(await fill()).toBeCloseTo(0.2);
    await page.emulateMedia({ reducedMotion: "reduce" });
    expect(await fill()).toBeLessThan(0.001);

    // Reviews are read when a run's review state changes, never on the output poll.
    const reads = () => page.evaluate(() => Number(localStorage.getItem("pytxo-preview-review-reads-v1") ?? "0"));
    const before = await reads();
    await page.waitForTimeout(3_200);
    expect(await reads()).toBe(before);

    // A verdict opens that run's Review; the wall applies nothing.
    await pane(page, "check-failed").locator("button.verdict").click();
    await expect(wall).toHaveCount(0);
    await expect(page.getByText("Combined check failed", { exact: true })).toBeVisible();
    await expect(page.getByText("task-1: npm test failed on the combined change")).toBeVisible();
  });

  test("presenting shows the wall alone, keeps approvals and Stop in reach, and Esc ends it", async ({ page }) => {
    const wall = page.getByTestId("fleet-wall");
    const html = page.locator("html");
    const chrome = [page.locator("aside.sidebar"), page.locator(".app-bar")];
    await expect(pane(page, "ready").locator("button.verdict")).toHaveAttribute("data-tone", "verified");
    const statSize = () => wall.locator('.stats [data-kind="ready"] dd').evaluate((element) => parseFloat(getComputedStyle(element).fontSize));
    const normalSize = await statSize();

    await wall.getByRole("button", { name: "Present", exact: true }).click();
    await expect(html).toHaveAttribute("data-presenting", "");
    for (const part of chrome) await expect(part).toBeHidden();
    await expect(page.locator(".titlebar-wrap")).toBeHidden();
    await expect(wall.getByRole("button", { name: "Run", exact: true })).toHaveCount(0);
    expect(await statSize()).toBeGreaterThan(normalSize);
    // The home folder reads as ~ while presenting.
    const output = pane(page, "ready").locator("button.tile").first().locator(".out");
    await expect(output).toContainText("writing ~\\fw\\ready\\src\\task-1.ts");
    await expect(wall.locator(".tile .out", { hasText: "C:\\Users\\preview" })).toHaveCount(0);

    await page.keyboard.press("Escape");
    await expect(html).not.toHaveAttribute("data-presenting", "");
    for (const part of chrome) await expect(part).toBeVisible();
    await expect(output).toContainText("writing C:\\Users\\preview\\fw\\ready\\src\\task-1.ts");

    // Pending approvals stay one click away, and that click ends presenting.
    await wall.getByRole("button", { name: "Present", exact: true }).click();
    const approvals = wall.getByRole("button", { name: /^\d+ approvals? waiting$/ });
    await expect(approvals).toBeVisible();
    await approvals.click();
    await expect(html).not.toHaveAttribute("data-presenting", "");
    await expect(page.getByRole("dialog", { name: "Approvals inbox" })).toBeVisible();
    await page.getByRole("button", { name: "Close approvals" }).click();

    // Stop is reached through a tile: opening its run ends presenting.
    await wall.getByRole("button", { name: "Present", exact: true }).click();
    await pane(page, "working").locator("button.tile").first().click();
    await expect(html).not.toHaveAttribute("data-presenting", "");
    await expect(page.getByRole("heading", { level: 1 })).toHaveText("Work in working");
    await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeVisible();
  });
});
