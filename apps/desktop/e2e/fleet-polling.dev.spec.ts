import { expect, test, type Page } from "@playwright/test";

async function mount(page: Page, checks = ["npm test"]) {
  await page.goto("/");
  await page.evaluate(async (verify) => {
    const svelteUrl = "/node_modules/svelte/src/index-client.js";
    const harnessUrl = "/e2e/fixtures/FleetHarness.svelte";
    const [{ mount }, { default: Harness }] = await Promise.all([import(svelteUrl), import(harnessUrl)]);
    const pending: { run: string; agent: string; cursor: number; resolve: (value: unknown[]) => void; reject: (error: Error) => void }[] = [];
    const target = document.createElement("main");
    target.style.cssText = "position:fixed;inset:0;background:white;z-index:9999";
    document.body.append(target);
    const harness = mount(Harness, { target, props: { checks: verify, backend: {
      readAgentEvents: (run: string, agent: string, _domain: string, cursor: number) => new Promise((resolve, reject) => pending.push({ run, agent, cursor, resolve, reject })),
    } } });
    Object.assign(window, { fleetTest: { harness, pending } });
  }, checks);
}

test("switching runs discards late reads, including reused worker IDs", async ({ page }) => {
  await mount(page);
  await expect.poll(() => page.evaluate(() => (window as any).fleetTest.pending.length)).toBe(2);
  await page.evaluate(() => {
    const { harness } = (window as any).fleetTest;
    harness.switchRun("run-b");
  });
  await expect.poll(() => page.evaluate(() => (window as any).fleetTest.pending.filter((read: any) => read.run === "run-b").length)).toBe(2);
  await page.evaluate(() => {
    for (const read of (window as any).fleetTest.pending.filter((item: any) => item.run === "run-a")) {
      read.resolve([{ id: 1, agent_id: read.agent, kind: "stdout", payload: "OLD RUN OUTPUT", ts: new Date().toISOString() }]);
    }
  });
  await page.waitForTimeout(1800);
  expect(await page.evaluate(() => (window as any).fleetTest.pending.length)).toBe(4);
  await page.evaluate(() => {
    for (const read of (window as any).fleetTest.pending.filter((item: any) => item.run === "run-b")) {
      if (read.cursor !== 0) throw new Error("Reused another run's cursor");
      read.resolve([{ id: 1, agent_id: read.agent, kind: "stdout", payload: "NEW RUN OUTPUT", ts: new Date().toISOString() }]);
    }
  });
  await expect(page.getByTestId("fleet-board")).toContainText("NEW RUN OUTPUT");
  await expect(page.getByTestId("fleet-board")).not.toContainText("OLD RUN OUTPUT");
});

test("one failed read cannot release the polling lock while its sibling is pending", async ({ page }) => {
  await mount(page);
  await expect.poll(() => page.evaluate(() => (window as any).fleetTest.pending.length)).toBe(2);
  await page.evaluate(() => (window as any).fleetTest.pending[0].reject(new Error("Disconnected")));
  await expect(page.getByText("Output unavailable. Retrying...")).toBeVisible();
  await page.waitForTimeout(1800);
  expect(await page.evaluate(() => (window as any).fleetTest.pending.length)).toBe(2);
  await page.evaluate(() => (window as any).fleetTest.pending[1].resolve([{ id: 1, agent_id: "two", kind: "stdout", payload: "SIBLING OUTPUT", ts: new Date().toISOString() }]));
  await expect.poll(() => page.evaluate(() => (window as any).fleetTest.pending.length)).toBe(4);
  await page.evaluate(() => {
    const reads = (window as any).fleetTest.pending;
    reads[2].resolve([{ id: 1, agent_id: "one", kind: "stdout", payload: "RECOVERED OUTPUT", ts: new Date().toISOString() }]);
    reads[3].resolve([]);
  });
  await expect(page.getByText("Output unavailable. Retrying...")).toHaveCount(0);
  await expect(page.getByText("SIBLING OUTPUT", { exact: true })).toHaveCount(1);
  await expect(page.getByText("RECOVERED OUTPUT", { exact: true })).toHaveCount(1);
});

test("completion reads final output without claiming unconfigured checks", async ({ page }) => {
  await mount(page, []);
  await expect.poll(() => page.evaluate(() => (window as any).fleetTest.pending.length)).toBe(2);
  await page.evaluate(() => {
    for (const read of (window as any).fleetTest.pending) read.resolve([]);
  });
  await page.evaluate(() => (window as any).fleetTest.harness.complete());
  await expect.poll(() => page.evaluate(() => (window as any).fleetTest.pending.length)).toBe(4);
  await page.evaluate(() => {
    for (const read of (window as any).fleetTest.pending.slice(2)) read.resolve([{ id: 1, agent_id: read.agent, kind: "stdout", payload: "FINAL OUTPUT", ts: new Date().toISOString() }]);
  });
  const board = page.getByTestId("fleet-board");
  await expect(board.getByText("FINAL OUTPUT", { exact: true })).toHaveCount(2);
  await expect(board.locator(".state")).toHaveText(["Completed", "Completed"]);
  await expect(board.locator(".facts")).toContainText("No task checks configured");
  await expect(board).not.toContainText("Checks passed");
});
