import { expect, test } from "@playwright/test";
import { workbenchSelection, rememberWorkbenchSelection } from "../src/lib/workbench-selection";
import { executionTopology } from "../src/lib/execution-topology";
import type { AgentDto, RoutingDisplayAttempt, RoutingDisplaySummary, RunReviewDto } from "../src/lib/types";
import { completeOnboarding } from "./helpers";

const task = (id: string, depends_on: string[] = []) => ({ task_id: id, depends_on, paths: [], agent: "codex", wave: 0, root: null, verify: [] });
const run = { id: "run", domain_id: "repo" };
const agent: AgentDto = { id: "worker", run_id: "run", domain_id: "repo", task_id: "a", wave: 0, status: "running", exit_code: null, root_id: null };

test("topology does not invent dependencies or associate foreign or ambiguous workers", () => {
  const plan: RunReviewDto["plan"] = { waves: [[task("a"), task("b")], [task("c", ["a", "missing"])]], warnings: [] };
  const map = executionTopology(plan, [agent, { ...agent, id: "foreign", domain_id: "other" }], run);
  expect(map.edges.map(edge => [edge.source.task.task_id, edge.target.task.task_id])).toEqual([["a", "c"]]);
  expect(map.nodes[0].agent?.id).toBe("worker");
  expect(map.nodes[1].agent).toBeUndefined();
  expect(executionTopology(plan, [agent, { ...agent, id: "ambiguous" }], run).nodes[0].agent).toBeUndefined();
  expect(executionTopology(null, [agent], run).nodes).toEqual([]);
});

test("routed topology resolves the current attempt by exact agent identity", () => {
  const decision = { selection: { kind: "selected" as const, role: "strong" as const }, reason: "strong_repair" as const, advice_status: "rules_fallback" as const };
  const attempt = (id: string, agentId: string, ordinal: number, state: RoutingDisplayAttempt["state"]): RoutingDisplayAttempt => ({
    attempt_id: id, agent_id: agentId, ordinal, predecessor_id: ordinal === 1 ? null : "attempt-1",
    state, role: ordinal === 1 ? "everyday" : "strong", decision, profile_id: "codex-strong",
    harness_id: "codex", billing_mode: "subscription", handoff_referenced: false,
    sealed_output_recorded: false, checks_receipt_recorded: false, usage_status: "unknown",
    ownership_released: state === "failed", admitted_at_ms: "1", updated_at_ms: "2",
  });
  const summary: RoutingDisplaySummary = {
    domain_id: "repo", run_id: "run", routing_revision: "7", mode: "shadow", cancelled: false,
    tasks: [{ task_id: "a", state: "active", dependency_task_ids: [], current_attempt_id: "attempt-2", winning_attempt_id: null,
      last_decision: decision, pre_admission: null, attempts: [attempt("attempt-1", "worker-1", 1, "failed"), attempt("attempt-2", "worker-2", 2, "running")] }],
  };
  const runWithRouting = { ...run, routing_revision: "7" };
  const first = { ...agent, id: "worker-1", status: "failed" };
  const second = { ...agent, id: "worker-2" };
  const plan: RunReviewDto["plan"] = { waves: [[task("a")]], warnings: [] };
  const current = executionTopology(plan, [first, second], runWithRouting, "horizontal", summary).nodes[0];
  expect(current.agent?.id).toBe("worker-2");
  expect(current.routingAttempt?.ordinal).toBe(2);
  expect(current.attemptCount).toBe(2);
  expect(executionTopology(plan, [first, second], runWithRouting, "horizontal", { ...summary, routing_revision: "6" }).nodes[0].agent).toBeUndefined();
  expect(executionTopology(plan, [first], runWithRouting, "horizontal", summary).nodes[0].agent).toBeUndefined();
  expect(executionTopology(plan, [{ ...second, task_id: "other" }], runWithRouting, "horizontal", summary).nodes[0].agent).toBeUndefined();
  expect(executionTopology(plan, [first, second], runWithRouting, "horizontal", null).nodes[0].agent).toBeUndefined();
  expect(executionTopology(plan, [first], { ...run, routing_revision: null }, "horizontal", null, true).nodes[0].agent).toBeUndefined();
});

test("stable layouts preserve one worker, branches, and 20 to 24 task plans", () => {
  const single = executionTopology({ waves: [[task("only")]], warnings: [] }, [], run);
  expect(single.nodes[0]).toMatchObject({ sceneX: single.width / 2, sceneY: single.height / 2 });

  const branching = { waves: [[task("plan")], [task("ui", ["plan"]), task("api", ["plan"])], [task("tests", ["ui", "api"])]], warnings: [] };
  const first = executionTopology(branching, [], run);
  const second = executionTopology(branching, [], run);
  expect(first.nodes.map(({ task, sceneX, sceneY }) => [task.task_id, sceneX, sceneY])).toEqual(second.nodes.map(({ task, sceneX, sceneY }) => [task.task_id, sceneX, sceneY]));
  expect(first.edges).toHaveLength(4);

  for (const waves of [[Array.from({ length: 20 }, (_, i) => task(String(i)))], Array.from({ length: 6 }, (_, wave) => Array.from({ length: 4 }, (_, i) => task(`${wave}-${i}`)))]) {
    const map = executionTopology({ waves, warnings: [] }, [], run);
    expect(map.compact).toBe(true);
    expect(map.nodes.map(node => node.task.task_id)).toEqual(waves.flat().map(task => task.task_id));
    expect(new Set(map.nodes.map(node => `${node.sceneX}:${node.sceneY}`)).size).toBe(map.nodes.length);
    for (const node of map.nodes) {
      expect(node.sceneX).toBeGreaterThanOrEqual(112);
      expect(node.sceneX).toBeLessThanOrEqual(map.width - 112);
      expect(node.sceneY).toBeGreaterThanOrEqual(50);
      expect(node.sceneY).toBeLessThanOrEqual(map.height - 50);
    }
  }

  const vertical = executionTopology(branching, [], run, "vertical");
  expect(vertical.orientation).toBe("vertical");
  expect(vertical.nodes.find(node => node.task.task_id === "plan")!.sceneY).toBeLessThan(vertical.nodes.find(node => node.task.task_id === "ui")!.sceneY);
});

for (const viewport of [{ width: 1920, height: 1080 }, { width: 1280, height: 720 }, { width: 860, height: 760 }, { width: 390, height: 760 }]) {
  test(`execution remains viewport locked at ${viewport.width}x${viewport.height}`, async ({ page }, testInfo) => {
    await completeOnboarding(page);
    await page.setViewportSize(viewport);
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.goto("/#/work");
    const map = page.getByTestId("execution-map");
    await expect(map.getByText("Requires plan", { exact: true })).toBeVisible();
    await expect(map.getByText("Active worker", { exact: true })).toHaveCount(1);
    await expect(map.locator(".commit-rail")).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Review changes", exact: true })).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    expect(await page.locator(".work-content").evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: testInfo.outputPath(`execution-${viewport.width}.png`) });
    await map.getByRole("button", { name: /^ui / }).focus();
    await page.keyboard.press("Enter");
    const summary = page.locator(".dock-panel:visible").filter({ hasText: "Recorded scope" });
    await expect(summary).toBeVisible();
    if ([1280, 390].includes(viewport.width)) await page.screenshot({ path: testInfo.outputPath(`execution-${viewport.width}-details.png`) });
    await page.keyboard.press("Escape");
    await expect(summary).toBeHidden();
    await expect(map.locator(".task-node.chosen")).toContainText("ui");
    await map.getByRole("button", { name: /^ui / }).click();
    await page.getByRole("button", { name: "Open output", exact: true }).click();
    await expect(page.getByRole("region", { name: "Recorded agent output", exact: true })).toBeVisible();
    if (viewport.width === 1280) {
      await expect(page.getByRole("tablist", { name: "right dock" })).toHaveCount(0);
      await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("ui");
      await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Output · ui");
      expect((await map.locator(".map-body").boundingBox())!.height).toBeGreaterThan(120);
    }
    if ([1280, 390].includes(viewport.width)) await page.screenshot({ path: testInfo.outputPath(`execution-${viewport.width}-output.png`) });
  });
}

test("canvas controls pan, zoom, fit, and switch to the complete list", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto("/#/work");
  const map = page.getByTestId("execution-map");
  const viewport = map.getByRole("application", { name: /Worker canvas/ });
  const scene = map.locator(".scene");
  const initial = await scene.getAttribute("style");
  await viewport.focus();
  await page.keyboard.press("+");
  await expect.poll(() => scene.getAttribute("style")).not.toBe(initial);
  const beforePan = await scene.getAttribute("style");
  const bounds = (await viewport.boundingBox())!;
  await page.mouse.move(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
  await page.mouse.wheel(0, 90);
  await expect.poll(() => scene.getAttribute("style")).not.toBe(beforePan);
  expect(await page.locator(".work-content").evaluate(element => element.scrollTop)).toBe(0);
  await map.getByRole("button", { name: "Fit", exact: true }).click();
  const canvasBounds = (await viewport.boundingBox())!;
  for (const node of await map.locator(".task-node").all()) {
    const nodeBounds = (await node.boundingBox())!;
    expect(nodeBounds.x).toBeGreaterThanOrEqual(canvasBounds.x - 1);
    expect(nodeBounds.y).toBeGreaterThanOrEqual(canvasBounds.y - 1);
    expect(nodeBounds.x + nodeBounds.width).toBeLessThanOrEqual(canvasBounds.x + canvasBounds.width + 1);
    expect(nodeBounds.y + nodeBounds.height).toBeLessThanOrEqual(canvasBounds.y + canvasBounds.height + 1);
  }
  await map.getByRole("button", { name: "List", exact: true }).click();
  await expect(map.locator(".ledger")).toBeVisible();
  await expect(map.locator(".ledger").getByRole("button")).toHaveCount(3);
  await map.getByRole("button", { name: "Canvas", exact: true }).click();
  await expect(viewport).toBeVisible();
});

test("Runs chooser stays clickable below the narrow command strip with a right dock", async ({ page }, testInfo) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Details", exact: true }).click();
  const chooser = page.locator(".run-reference");
  await chooser.locator("summary").click();
  const item = chooser.getByRole("tab").first();
  await expect(item).toBeVisible();
  expect(await item.evaluate(element => {
    const bounds = element.getBoundingClientRect();
    return element.contains(document.elementFromPoint(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2));
  })).toBe(true);
  await page.screenshot({ path: testInfo.outputPath("runs-chooser-right-dock.png") });
  await item.click();
  await expect(chooser).not.toHaveAttribute("open");
  await chooser.locator("summary").focus();
  await page.keyboard.press("Enter");
  await page.keyboard.press("Tab");
  await expect(item).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(chooser).not.toHaveAttribute("open");
  expect(await page.locator(".work-content").evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);
});

test("returning from List restores the camera and wheel panning", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 720 });
  await page.goto("/#/work");
  const map = page.getByTestId("execution-map");
  const scene = map.locator(".scene");
  await map.getByRole("button", { name: "Zoom in", exact: true }).click();
  const transform = await scene.evaluate(element => element.style.transform);
  expect(transform).not.toBe("");
  await map.getByRole("button", { name: "List", exact: true }).click();
  await expect(scene).toHaveCount(0);
  await map.getByRole("button", { name: "Canvas", exact: true }).click();
  await expect.poll(() => scene.evaluate(element => element.style.transform)).toBe(transform);
  const bounds = (await map.getByRole("application", { name: /Worker canvas/ }).boundingBox())!;
  await page.mouse.move(bounds.x + bounds.width / 2, bounds.y + bounds.height / 2);
  await page.mouse.wheel(0, 90);
  await expect.poll(() => scene.evaluate(element => element.style.transform)).not.toBe(transform);
  expect(await page.locator(".work-content").evaluate(element => element.scrollTop)).toBe(0);
});

for (const manual of [false, true]) {
  test(`delayed plan ${manual ? "preserves a manually positioned camera" : "automatically fits when it arrives"}`, async ({ page }) => {
    await completeOnboarding(page, {
      "pytxo-preview-topology-v1": "24",
      "pytxo-preview-delayed-plan-v1": "1",
    });
    await page.setViewportSize({ width: 1280, height: 720 });
    await page.goto("/#/work");
    const map = page.getByTestId("execution-map");
    await expect(map.getByText("Task relationships unavailable", { exact: true })).toBeVisible();
    const zoom = map.getByRole("button", { name: "Reset zoom to 100 percent", exact: true });
    if (manual) await map.getByRole("button", { name: "Zoom in", exact: true }).click();
    const before = await zoom.textContent();
    await expect(map.locator(".task-node")).toHaveCount(24);
    if (manual) {
      await expect(zoom).toHaveText(before!);
    } else {
      const automatic = await map.locator(".scene").evaluate(element => element.style.transform);
      await map.getByRole("button", { name: "Fit", exact: true }).click();
      await expect.poll(() => map.locator(".scene").evaluate(element => element.style.transform)).toBe(automatic);
    }
  });
}

for (const width of [1280, 390]) {
  test(`minimap navigates dense recorded workers by pointer and keyboard at ${width}px`, async ({ page }, testInfo) => {
    await completeOnboarding(page, { "pytxo-preview-topology-v1": "24" });
    await page.setViewportSize({ width, height: 760 });
    await page.goto("/#/work");
    const map = page.getByTestId("execution-map");
    await expect(map.locator(".task-node")).toHaveCount(24);
    const minimap = map.getByRole("button", { name: /^Navigate canvas minimap/ });
    await expect(minimap).toBeInViewport();
    const scene = map.locator(".scene");
    const initial = await scene.evaluate(element => element.style.transform);
    await minimap.click({ position: { x: 15, y: 15 } });
    await expect.poll(() => scene.evaluate(element => element.style.transform)).not.toBe(initial);
    const clicked = await scene.evaluate(element => element.style.transform);
    const frameX = await map.locator(".minimap-viewport").getAttribute("x");
    await minimap.press("ArrowRight");
    await expect.poll(() => scene.evaluate(element => element.style.transform)).not.toBe(clicked);
    await expect.poll(() => map.locator(".minimap-viewport").getAttribute("x")).not.toBe(frameX);
    await minimap.press("Enter");
    const canvasBounds = (await map.locator(".canvas-viewport").boundingBox())!;
    const selectedBounds = (await map.locator(".task-node.chosen").boundingBox())!;
    expect(Math.abs(selectedBounds.x + selectedBounds.width / 2 - canvasBounds.x - canvasBounds.width / 2)).toBeLessThan(2);
    expect(Math.abs(selectedBounds.y + selectedBounds.height / 2 - canvasBounds.y - canvasBounds.height / 2)).toBeLessThan(2);
    expect(await page.locator(".work-content").evaluate(element => element.scrollTop)).toBe(0);
    await expect(page.locator(".dock-panel:visible")).toHaveCount(0);
    await minimap.press("f");
    await expect(minimap).toBeFocused();
    await page.screenshot({ path: testInfo.outputPath(`minimap-${width}.png`) });
  });
}

for (const width of [1280, 390]) {
  test(`Fit exposes every dense node and preserves a panned overview at ${width}px`, async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-topology-v1": "24" });
    await page.setViewportSize({ width, height: 760 });
    await page.goto("/#/work");
    const map = page.getByTestId("execution-map");
    await expect(map.locator(".task-node")).toHaveCount(24);
    await map.getByRole("button", { name: "Fit", exact: true }).click();
    const viewport = map.locator(".canvas-viewport");
    const scene = map.locator(".scene");
    await expect.poll(async () => scene.evaluate(element => new DOMMatrix(getComputedStyle(element).transform).a)).toBeLessThan(.55);
    await expect(scene).toHaveClass(/overview/);
    if (width === 390) await expect(scene).toHaveClass(/micro-overview/);
    else await expect(scene).not.toHaveClass(/micro-overview/);
    await expect(map.locator(".node-overview").first()).toBeVisible();
    await expect(map.locator(".node-copy").first()).toBeHidden();
    const bounds = (await viewport.boundingBox())!;
    for (const node of await map.locator(".task-node").all()) {
      const box = (await node.boundingBox())!;
      expect(box.x).toBeGreaterThanOrEqual(bounds.x + 47);
      expect(box.y).toBeGreaterThanOrEqual(bounds.y + 47);
      expect(box.x + box.width).toBeLessThanOrEqual(bounds.x + bounds.width - 47);
      expect(box.y + box.height).toBeLessThanOrEqual(bounds.y + bounds.height - 47);
    }
    const overview = await scene.getAttribute("style");
    await map.getByRole("button", { name: "Zoom out", exact: true }).click();
    expect(await scene.getAttribute("style")).toBe(overview);
    await viewport.focus();
    await page.keyboard.press("ArrowRight");
    const panned = await scene.getAttribute("style");
    await map.getByRole("button", { name: "List", exact: true }).click();
    await map.getByRole("button", { name: "Canvas", exact: true }).click();
    await expect.poll(() => scene.getAttribute("style")).toBe(panned);
    await map.getByRole("button", { name: "Zoom in", exact: true }).click();
    await expect.poll(async () => scene.evaluate(element => new DOMMatrix(getComputedStyle(element).transform).a)).toBeGreaterThanOrEqual(.55);
    expect(await scene.evaluate(element => new DOMMatrix(getComputedStyle(element).transform).a)).toBeLessThanOrEqual(1.6);
    await expect(scene).not.toHaveClass(/overview/);
    await expect(scene).not.toHaveClass(/micro-overview/);
    await expect(map.locator(".node-copy").first()).toBeVisible();
  });
}

test("dense output stays dormant until opened and mounts at most 120 rows", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-topology-v1": "24",
    "pytxo-preview-output-events-v1": "600",
    "pytxo-preview-observe-polls-v1": "1",
  });
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto("/#/work");
  const map = page.getByTestId("execution-map");
  await expect(map.locator(".task-node")).toHaveCount(24);
  await expect(map.getByLabel("Canvas minimap", { exact: true })).toBeVisible();
  expect(await page.evaluate(() => localStorage.getItem("pytxo-preview-agent-event-reads-v1"))).toBeNull();
  await map.getByRole("button", { name: /^ui / }).click();
  await expect(page.locator(".dock-panel:visible")).toContainText("Recorded scope");
  const reviewReads = Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-review-reads-v1") ?? "0"));
  await page.waitForTimeout(2100);
  expect(Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-review-reads-v1") ?? "0"))).toBe(reviewReads);
  expect(await page.evaluate(() => localStorage.getItem("pytxo-preview-agent-event-reads-v1"))).toBeNull();
  await page.getByRole("button", { name: "Open output", exact: true }).click();
  await expect.poll(async () => Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-agent-event-reads-v1") ?? "0"))).toBeGreaterThan(0);
  await expect(page.getByRole("region", { name: "Recorded agent output", exact: true })).toBeVisible();
  const mountedRows = await page.locator(".dock-panel:visible .event").count();
  expect(mountedRows).toBeGreaterThan(0);
  expect(mountedRows).toBeLessThanOrEqual(120);
  // A closed output tab must release its timer as well as its rendered rows.
  // Wait beyond one polling period so an accidentally retained timer is caught.
  await page.getByRole("button", { name: /^Close Output .* view$/ }).click();
  await expect(page.getByRole("region", { name: "Recorded agent output", exact: true })).toHaveCount(0);
  await expect(page.locator(".dock-panel .event")).toHaveCount(0);
  const readsAfterClose = Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-agent-event-reads-v1") ?? "0"));
  await page.waitForTimeout(2200);
  expect(Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-agent-event-reads-v1") ?? "0"))).toBe(readsAfterClose);
});

test("idle polling reloads the full snapshot only when the workspace catalog changes", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-observe-polls-v1": "1" });
  await page.goto("/#/work");
  await expect(page.getByTestId("execution-map")).toBeVisible();
  await page.waitForTimeout(1000);
  const before = Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-snapshot-reads-v1") ?? "0"));

  // The preview contains a running worker, so the active delta interval is
  // 2.5 seconds. An unchanged catalog must not trigger a full snapshot.
  await page.waitForTimeout(3000);
  expect(Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-snapshot-reads-v1") ?? "0"))).toBe(before);

  await page.evaluate(() => localStorage.setItem("pytxo-preview-catalog-fingerprint-v1", "changed-catalog"));
  await expect.poll(
    async () => Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-snapshot-reads-v1") ?? "0")),
    { timeout: 5000 },
  ).toBeGreaterThan(before);
});

for (const viewport of [{ width: 1920, height: 1080 }, { width: 1280, height: 720 }, { width: 860, height: 760 }, { width: 390, height: 760 }]) {
  test(`New Work keeps draft and planned states inside ${viewport.width}x${viewport.height}`, async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "long-mission" });
    await page.setViewportSize(viewport);
    await page.goto("/#/flow");
    const route = page.locator(".work-content");
    expect(await route.evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);
    await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeInViewport();
    await expect(page.locator(".flow-history")).toBeVisible();
    await page.getByLabel("What should Pytxo do?").fill("Keep New Work fixed while preparing a reviewable plan.");
    await page.getByRole("button", { name: "Build plan", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Review plan", exact: true })).toBeVisible();
    await expect(page.locator(".composer-panel button").filter({ hasText: /^Build plan$/ })).toHaveCount(1);
    await expect(page.getByRole("button", { name: "Run", exact: true })).toBeInViewport();
    expect(await route.evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);
  });
}

test("New Work planning, blocked, and error states retain pane-owned scrolling", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-delay-v1": "1" });
  await page.setViewportSize({ width: 860, height: 760 });
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Exercise the fixed planning state.");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByText("Building plan…", { exact: true })).toBeVisible();
  expect(await page.locator(".work-content").evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);
  await expect(page.getByRole("heading", { name: "Review plan", exact: true })).toBeVisible();

  await page.evaluate(() => localStorage.removeItem("pytxo-preview-flow-delay-v1"));
  await page.getByLabel("Agent CLI").selectOption("claude");
  await page.getByRole("button", { name: "Request", exact: true }).click();
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Plan blocked", exact: true })).toBeVisible();
  expect(await page.locator(".work-content").evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);

  await page.evaluate(() => localStorage.setItem("pytxo-preview-flow-error-v1", "1"));
  await page.getByRole("button", { name: "Request", exact: true }).click();
  await page.getByLabel("Agent CLI").selectOption("codex");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Preview failed while checking the selected workspace");
  expect(await page.locator(".work-content").evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);
});

test("@performance dense canvas gestures and successive selections avoid browser long tasks", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-topology-v1": "24", "pytxo-preview-observe-polls-v1": "1" });
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto("/#/work");
  await page.evaluate(() => {
    (window as any).__pytxoLongTasks = [];
    (window as any).__pytxoPerformanceMarks = [{ label: "observer-ready", time: performance.now() }];
    (window as any).__pytxoLongTaskObserver = new PerformanceObserver(list => {
      (window as any).__pytxoLongTasks.push(...list.getEntries().map(entry => ({
        duration: entry.duration,
        startTime: entry.startTime,
      })));
    });
    (window as any).__pytxoLongTaskObserver.observe({ entryTypes: ["longtask"] });
    const recordInteraction = (event: Event) => {
      const target = event.target instanceof Element ? event.target.closest("button, [role='application']") : null;
      (window as any).__pytxoPerformanceMarks.push({
        label: `${event.type}:${target?.getAttribute("aria-label") ?? target?.textContent?.trim().slice(0, 48) ?? "canvas"}`,
        time: performance.now(),
      });
    };
    document.addEventListener("pointerdown", recordInteraction, true);
    document.addEventListener("pointerup", recordInteraction, true);
    document.addEventListener("click", recordInteraction, true);
  });
  const map = page.getByTestId("execution-map");
  const viewport = map.getByRole("application", { name: /Worker canvas/ });
  const bounds = (await viewport.boundingBox())!;
  await page.mouse.move(bounds.x + bounds.width * .72, bounds.y + bounds.height * .72);
  await page.mouse.down();
  await page.mouse.move(bounds.x + bounds.width * .58, bounds.y + bounds.height * .62, { steps: 8 });
  await page.mouse.up();
  await map.getByRole("button", { name: "Zoom in", exact: true }).click();
  // Panning deliberately moves the first wave outside the viewport. Reframe
  // before the selection sequence instead of clicking through the sidebar.
  await map.getByRole("button", { name: "Fit", exact: true }).click();
  for (let index = 0; index < 10; index++) await map.locator(".task-node button").nth(index).click();
  await page.waitForTimeout(100);
  const performance = await page.evaluate(() => {
    (window as any).__pytxoLongTaskObserver.disconnect();
    return {
      longTasks: (window as any).__pytxoLongTasks as { duration: number; startTime: number }[],
      marks: (window as any).__pytxoPerformanceMarks as { label: string; time: number }[],
    };
  });
  expect(performance.longTasks.filter(entry => entry.duration > 50), JSON.stringify(performance, null, 2)).toEqual([]);
  expect(Number(await page.evaluate(() => localStorage.getItem("pytxo-preview-review-reads-v1") ?? "0"))).toBe(1);
});

for (const state of ["stale", "recovery_required", "unavailable"]) {
  test(`the aperture does not imply Apply in ${state}`, async ({ page }, testInfo) => {
    await completeOnboarding(page, { "pytxo-preview-review-state-v1": state, "pytxo-preview-candidate-check-v1": "passed" });
    await page.setViewportSize({ width: 860, height: 560 });
    await page.goto("/#/run-review");
    await expect(page.locator("#run-review-title")).toBeVisible();
    await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeDisabled();
    await expect(page.locator(".repository-boundary.confirmed")).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeInViewport();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: testInfo.outputPath(`boundary-${state}.png`) });
  });
}

test("light theme and the empty composer retain the same operational hierarchy", async ({ page }, testInfo) => {
  await completeOnboarding(page, { "pytxo-deck-theme": "light", "pytxo-preview-review-state-v1": "ready", "pytxo-preview-candidate-check-v1": "passed" });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  for (const route of ["flow", "work", "run-review", "history"]) {
    await page.goto(`/#/${route}`);
    await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", "light");
    await expect(page.locator(".content h1").first()).toBeVisible();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: testInfo.outputPath(`light-${route}.png`) });
  }
});

test("an in-flight Apply is neither a confirmed outcome nor a recovery failure", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-review-state-v1": "applying", "pytxo-preview-candidate-check-v1": "passed" });
  await page.goto("/#/run-review");
  await expect(page.locator(".repository-boundary")).toContainText("Apply in progress · outcome not confirmed");
  await expect(page.locator(".repository-boundary.confirmed")).toHaveCount(0);
  await expect(page.locator(".repository-boundary")).not.toContainText("needs reconciliation");
});


test("task selection highlights only recorded connections and survives Review", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/work");
  const map = page.getByTestId("execution-map");
  await map.getByRole("button", { name: /^ui / }).click();
  await expect(map.locator(".task-node.chosen")).toContainText("ui");
  await expect(map.locator("path.highlighted")).toHaveCount(2);
  await expect(map.getByRole("region", { name: "Selected task relationships" })).toContainText("Needs");
  await page.getByRole("button", { name: "Review changes", exact: true }).click();
  await expect(page.locator("#run-review-title")).toContainText("Changes in");
  await expect(page.getByRole("button", { name: "Inspection tools" })).toHaveAttribute("aria-expanded", "false");
  await page.getByRole("button", { name: "Back to Work", exact: true }).click();
  await expect(page.locator(".task-node.chosen")).toContainText("ui");
});

test("focused Review remembers its file and presents exact contents on neutral surfaces", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-review-state-v1": "ready", "pytxo-preview-candidate-check-v1": "passed" });
  await page.goto("/#/run-review");
  await expect(page.locator(".diff-side .text-content")).toHaveCount(2);
  const colors = await page.locator(".diff-side .text-content").evaluateAll(nodes => nodes.map(node => getComputedStyle(node).backgroundColor));
  expect(new Set(colors).size).toBe(1);
  const file = page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/tests/skeleton.rs", exact: true });
  await file.click();
  await expect(file).toHaveAttribute("aria-pressed", "true");
  await page.getByRole("button", { name: "Back to Work", exact: true }).click();
  await page.getByRole("button", { name: "Review changes", exact: true }).click();
  await expect(file).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator(".comparison-context")).toContainText("skeleton.rs");
});


test("inspection context is scoped to both run and repository", () => {
  rememberWorkbenchSelection("same-run", "one", { taskId: "selected", path: "src/one.ts" });
  expect(workbenchSelection("same-run", "one")).toEqual({ taskId: "selected", path: "src/one.ts" });
  expect(workbenchSelection("same-run", "two")).toEqual({});
  expect(workbenchSelection("other-run", "one")).toEqual({});
});
