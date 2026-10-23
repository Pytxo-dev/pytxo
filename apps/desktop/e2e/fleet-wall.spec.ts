import { expect, test } from "@playwright/test";
import { fleetLinks, fleetWall, runKey, wallLayout } from "../src/lib/fleet-wall";
import type { AgentDto, CatalogEntryStatus, FleetNodeDto, RunDto } from "../src/lib/types";
import { completeOnboarding } from "./helpers";

const run = (domain_id: string, id: string, status: string, started_at: string) =>
  ({ id, domain_id, repo_root: domain_id, status, started_at }) as unknown as RunDto;
const agent = (domain_id: string, run_id: string, id: string, task_id: string, status: string, exit_code: number | null = null, wave = 0) =>
  ({ id, domain_id, run_id, task_id, wave, status, exit_code, root_id: null, launcher: null }) as AgentDto;
const domain = (domain_id: string, repo_root: string) => ({ domain_id, repo_root }) as CatalogEntryStatus;

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
    await expect(wall.getByRole("region", { name: `${repo}: ${count} agents` }).getByRole("button")).toHaveCount(count);
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
  await expect(wall.getByRole("button", { name: /^fw1 task-7: done/ })).toHaveAttribute("data-tone", "verified");

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
