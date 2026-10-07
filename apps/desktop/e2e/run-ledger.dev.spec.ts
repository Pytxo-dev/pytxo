import { expect, test } from "@playwright/test";

test("List uses readable task descriptions while retaining identity and keyboard navigation", async ({ page }) => {
  await page.goto("/");
  await page.evaluate(async () => {
    const svelteUrl = "/node_modules/svelte/src/index-client.js";
    const ledgerUrl = "/src/components/desktop2/RunLedger.svelte";
    const [{ mount }, { default: Ledger }] = await Promise.all([import(svelteUrl), import(ledgerUrl)]);
    const target = document.createElement("main");
    target.id = "ledger-test";
    target.style.cssText = "position:fixed;inset:0;background:white;z-index:9999;padding:24px";
    document.body.append(target);
    const agents = [0, 1].map(index => ({
      id: `worker-${index}`, run_id: "run", domain_id: "domain", task_id: `mission-${index}`,
      wave: index, status: "completed", exit_code: 0, started_at: null, finished_at: null,
      launcher: { id: "codex", display_name: "OpenAI Codex" },
    }));
    mount(Ledger, { target, props: { agents, inlineInspector: false,
      taskDescriptions: { "mission-0": "Add readable risk summaries for network and destructive command changes" },
      onSelect: (id: string) => target.dataset.selected = id,
    } });
  });
  const ledger = page.locator("#ledger-test");
  await expect(ledger.locator(".task")).toHaveText([
    "Add readable risk summaries for network and destructive command changes", "mission-1",
  ]);
  const first = ledger.getByRole("button", { name: /Add readable risk summaries.*task mission-0, worker-0/ });
  await first.focus();
  await first.press("ArrowDown");
  await expect(ledger).toHaveAttribute("data-selected", "worker-1");
  await expect(ledger.getByRole("button", { name: /task mission-1, worker-1/ })).toBeFocused();
  await page.setViewportSize({ width: 960, height: 700 });
  expect(await ledger.evaluate(element => element.scrollWidth <= element.clientWidth)).toBe(true);
  await expect(ledger.locator(".task").first()).toHaveAttribute("title", /mission-0/);
});
