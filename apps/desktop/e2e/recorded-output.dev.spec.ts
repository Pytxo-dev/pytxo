import { expect, test } from "@playwright/test";

test("Output joins recorded soft wraps while Raw text preserves separate payloads", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(async () => {
    const svelteUrl = "/node_modules/svelte/src/index-client.js";
    const dockUrl = "/src/components/desktop2/DockInspection.svelte";
    const [{ mount }, { default: Dock }] = await Promise.all([import(svelteUrl), import(dockUrl)]);
    const target = document.createElement("main");
    target.id = "output-test";
    target.style.cssText = "position:fixed;inset:0;background:white;z-index:9999";
    document.body.append(target);
    mount(Dock, { target, props: {
      view: { kind: "agent-output", title: "Output", runId: "run", domainId: "domain", agentId: "worker" },
      snapshot: { runs: [], agents: [] }, visible: true, onReview: () => {}, onOpenApprovals: () => {},
      backend: { readAgentEvents: (_run: string, _agent: string, _domain: string, after: number) => Promise.resolve(after ? [] : [
        { id: 1, agent_id: "worker", kind: "stdout", ts: "", payload: " ".repeat(65) + "No risk is foun" },
        { id: 2, agent_id: "worker", kind: "stdout", ts: "", payload: "\x1b[23;80Hnd." },
        { id: 3, agent_id: "worker", kind: "stdout", ts: "", payload: "\x1b]52;c;hidden\x07Next record" },
      ]) },
    } });
  });
  const dock = page.locator("#output-test");
  const output = dock.getByRole("region", { name: "Recorded agent output", exact: true });
  await expect(output.locator("pre")).toHaveText("No risk is found.\nNext record");
  await expect(output).not.toContainText("hidden");
  await dock.getByRole("button", { name: "Raw text", exact: true }).click();
  await expect(output.locator("pre")).toHaveCount(3);
  await expect(output.locator("pre").nth(1)).toHaveText("\x1b[23;80Hnd.");
  await dock.getByRole("button", { name: "Plain text", exact: true }).click();
  await expect(output.locator("pre")).toHaveText("No risk is found.\nNext record");
});
