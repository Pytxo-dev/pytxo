import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("routed canvas keeps an admitted retry distinct from an unrelated worker row", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-routing-summary-v1": "1", "pytxo-preview-approval-scope-v1": "foreign-run" });
  await page.goto("/#/work");
  const canvas = page.getByTestId("execution-map");
  const ui = canvas.locator('.task-node button[aria-label^="ui "]');
  await expect(ui).toContainText("Attempt 2 · Strong · Admitted");
  await expect(ui).toContainText("Worker not recorded");
  await expect(page.locator(".aperture-glyph")).toHaveAttribute("data-tone", "unknown");
  await expect(page.locator(".work-summary")).toHaveText("Attempt 2 is admitted. Its worker has not been recorded yet.");
  await ui.click();
  await expect(page.locator(".dock-panel:visible").getByText("Agent desktop", { exact: true })).toHaveCount(0);
});

test("routed canvas opens the exact current retry and List retains both workers", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-routing-summary-v1": "1",
    "pytxo-preview-routed-retry-workers-v1": "1",
    "pytxo-preview-approval-scope-v1": "foreign-run",
  });
  await page.goto("/#/work");
  const canvas = page.getByTestId("execution-map");
  const ui = canvas.locator('.task-node button[aria-label^="ui "]');
  await expect(ui).toContainText("Attempt 2 · Strong · Running");
  await expect(page.locator(".aperture-glyph")).toHaveAttribute("data-tone", "active");
  await expect(canvas.locator('.task-node button[aria-label^="tests "]')).toHaveAttribute("aria-label", /No worker attempt recorded/);
  await page.screenshot({ path: test.info().outputPath("routing-retry-canvas.png"), fullPage: true });
  await ui.click();
  const inspector = page.locator(".dock-panel:visible");
  await inspector.locator(".source > summary").click();
  await expect(inspector).toContainText("Agent desktop-2");
  await expect(inspector).not.toContainText("Agent desktop-1");
  await canvas.getByRole("button", { name: "List", exact: true }).click();
  await expect(canvas.getByRole("button", { name: /ui, desktop-1,/ })).toBeVisible();
  await expect(canvas.getByRole("button", { name: /ui, desktop-2,/ })).toBeVisible();
});

test("routed Work and History show durable attempts without confusing them with Apply", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-routing-summary-v1": "1" });
  await page.goto("/#/work");
  const workRecord = page.getByRole("region", { name: "Routing and worker attempts" });
  await expect(workRecord.locator("summary")).toContainText("3 tasks · 3 attempts · Shadow mode");
  await expect(workRecord.locator("summary")).toContainText("Recorded routing state");
  await expect(workRecord.locator("summary")).toContainText("Capacity: tests · Everyday selected · no attempt admitted");
  await expect(workRecord.locator("summary")).toContainText("Unsettled attempt: ui · attempt 2 · Strong via codex · Admitted");
  await page.screenshot({ path: test.info().outputPath("routing-work-collapsed.png"), fullPage: true });
  await workRecord.locator("summary").click();
  await expect.poll(() => workRecord.evaluate((element) => element.scrollHeight > element.clientHeight)).toBe(true);
  expect((await page.getByTestId("execution-map").boundingBox())!.height).toBeGreaterThan(120);
  await workRecord.focus();
  await page.keyboard.press("PageDown");
  await expect.poll(() => workRecord.evaluate((element) => element.scrollTop)).toBeGreaterThan(0);
  await expect(workRecord).toContainText("Worker attempt 1");
  await expect(workRecord).toContainText("Worker attempt 2");
  await expect(workRecord).toContainText("Winner recorded");
  await expect(workRecord).toContainText("Shadow recorded");
  await expect(workRecord).toContainText("After plan · Succeeded · winning attempt recorded");
  await expect(workRecord).toContainText("Pre-admission decision for attempt 1: Selected · Everyday");
  await expect(workRecord).toContainText("Earlier pre-admission decision for attempt 2: Selected · Strong");
  await expect(workRecord).toContainText("A later decision admitted attempt 2");
  await expect(workRecord).toContainText("Not admitted · Capacity unavailable; no attempt admitted for this decision");
  await workRecord.locator(".task").last().scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("routing-pre-admission.png") });
  await expect(workRecord).not.toContainText("Apply recorded");
  const uiAttempts = workRecord.getByRole("list", { name: "Worker attempts for ui" });
  await expect(uiAttempts.getByRole("listitem").nth(0)).toContainText("Mechanical everyday");
  await expect(uiAttempts.getByRole("listitem").nth(1)).toContainText("Strong repair");
  await expect(uiAttempts.getByRole("listitem").nth(1)).toContainText("Follows attempt 1 (route-ui-1)");
  await expect(uiAttempts.getByRole("listitem").nth(0)).toContainText("Checker receipt recorded");
  await expect(uiAttempts.getByRole("listitem").nth(0)).toContainText("Ownership released");
  await expect(uiAttempts.getByRole("listitem").nth(1)).toContainText("Ownership not released");
  await page.screenshot({ path: test.info().outputPath("routing-work.png"), fullPage: true });

  await page.setViewportSize({ width: 860, height: 700 });
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-8f2c"][data-domain-id="pytxo"]').click();
  const historyRecord = page.getByRole("region", { name: "Routing and worker attempts" });
  await expect(historyRecord.locator("summary")).toContainText("Recorded routing state");
  await expect(historyRecord.locator("summary")).toContainText("Capacity: tests · Everyday selected · no attempt admitted");
  await historyRecord.scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("routing-history-collapsed-860.png"), fullPage: true });
  await historyRecord.locator("summary").click();
  await expect(historyRecord).toContainText("Worker attempt 2");
  await expect(page.locator(".recorded-trace")).toContainText("Apply record");
  await page.screenshot({ path: test.info().outputPath("routing-history-860.png"), fullPage: true });
  await page.getByRole("button", { name: "Back to runs" }).click();
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await expect(historyRecord).toHaveCount(0);
});

test("History discards a delayed summary for the same run ID in another domain", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-routing-summary-v1": "1",
    "pytxo-preview-routing-same-id-v1": "1",
    "pytxo-preview-routing-delay-v1": "1",
  });
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-8f2c"][data-domain-id="signal-lab"]').click();
  const record = page.getByRole("region", { name: "Routing and worker attempts" });
  await expect(record.locator("summary")).not.toContainText("Current:");
  await expect(record.locator("summary")).not.toContainText("Capacity:");
  await record.locator("summary").click();
  await expect(record).toContainText("other-domain-task");
  await page.waitForTimeout(1100);
  await expect(record).toContainText("other-domain-task");
  await expect(record).not.toContainText("Worker attempt 2");
  await expect(record).not.toContainText("plan");
  await expect(record.locator("summary")).not.toContainText("Unsettled attempt:");
  await expect(record.locator("summary")).not.toContainText("Capacity:");
});

test("cancelled routing keeps old capacity and unsettled attempts historical", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-routing-summary-v1": "1",
    "pytxo-preview-routing-cancelled-v1": "1",
  });
  await page.goto("/#/work");
  const record = page.getByRole("region", { name: "Routing and worker attempts" });
  await expect(record.locator("summary")).toContainText("Cancellation recorded · attempt settlement may still be pending");
  await expect(record.locator("summary")).not.toContainText("Capacity:");
  await expect(record.locator("summary")).not.toContainText("Unsettled attempt:");
  await record.locator("summary").click();
  await expect(record).toContainText("Cancellation recorded for this routing mission.");
  await expect(record).toContainText("Not admitted · Capacity unavailable; no attempt admitted for this decision");
  await expect(record).toContainText("Worker attempt 2");
});

test("a later policy block supersedes an older capacity stop", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-routing-summary-v1": "1",
    "pytxo-preview-routing-later-block-v1": "1",
  });
  await page.goto("/#/work");
  const record = page.getByRole("region", { name: "Routing and worker attempts" });
  await expect(record.locator("summary")).toContainText("Recorded routing state");
  await expect(record.locator("summary")).not.toContainText("Capacity:");
  await record.locator("summary").click();
  await expect(record).toContainText("Last admitted or blocked decision: Blocked · Budget exhausted");
  await expect(record).toContainText("Not admitted · Capacity unavailable; no attempt admitted for this decision");
});

test("routing read errors fail closed", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-routing-summary-v1": "1", "pytxo-preview-routing-error-v1": "1" });
  await page.goto("/#/work");
  const record = page.getByRole("region", { name: "Routing and worker attempts" });
  await record.locator("summary").click();
  await expect(record.getByRole("alert")).toContainText("routing read unavailable");
  await expect(record.locator("summary")).not.toContainText("Unsettled attempt:");
  await expect(record.locator("summary")).not.toContainText("Capacity:");
  await expect(record).not.toContainText("Worker attempt 1");
  const ui = page.getByTestId("execution-map").locator('.task-node button[aria-label^="ui "]');
  await expect(ui).toContainText("Routing record unavailable");
  await ui.click();
  await expect(page.getByRole("complementary", { name: "Selected worker details" })).toHaveCount(0);
  await page.evaluate(() => localStorage.removeItem("pytxo-preview-routing-error-v1"));
  await record.getByRole("button", { name: "Retry routing record" }).click();
  await expect(record.locator("summary")).toContainText("3 tasks · 3 attempts");
  await expect(ui).toContainText("Attempt 2 · Strong · Admitted");
  await page.evaluate(() => localStorage.setItem("pytxo-preview-routing-error-v1", "1"));
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-8f2c"][data-domain-id="pytxo"]').click();
  const historyRecord = page.getByRole("region", { name: "Routing and worker attempts" });
  await historyRecord.locator("summary").click();
  await expect(historyRecord.getByRole("alert")).toContainText("routing read unavailable");
  await page.evaluate(() => localStorage.removeItem("pytxo-preview-routing-error-v1"));
  await historyRecord.getByRole("button", { name: "Retry routing record" }).click();
  await expect(historyRecord.locator("summary")).toContainText("3 tasks · 3 attempts");
});

test("stale routing revisions fail closed", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-routing-summary-v1": "1", "pytxo-preview-routing-stale-revision-v1": "1" });
  await page.goto("/#/work");
  const record = page.getByRole("region", { name: "Routing and worker attempts" });
  await record.locator("summary").click();
  await expect(record.getByRole("alert")).toContainText("could not be matched");
  await expect(record.locator("summary")).not.toContainText("Unsettled attempt:");
  await expect(record.locator("summary")).not.toContainText("Capacity:");
  await expect(record).not.toContainText("Worker attempt 1");
  const ui = page.getByTestId("execution-map").locator('.task-node button[aria-label^="ui "]');
  await expect(ui).toContainText("Routing record unavailable");
  await ui.click();
  await expect(page.getByRole("complementary", { name: "Selected worker details" })).toHaveCount(0);
});

test("a routing identity read error cannot promote a lone agent as a legacy worker", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-routing-summary-v1": "1",
    "pytxo-preview-routing-revision-error-v1": "1",
    "pytxo-preview-approval-scope-v1": "foreign-run",
  });
  await page.goto("/#/work");
  const ui = page.getByTestId("execution-map").locator('.task-node button[aria-label^="ui "]');
  await expect(ui).toContainText("Routing identity unavailable");
  await expect(page.getByRole("alert")).toContainText("Routing identity could not be checked");
  await ui.click();
  await expect(page.getByRole("complementary", { name: "Selected worker details" })).toHaveCount(0);
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-8f2c"][data-domain-id="pytxo"]').click();
  await expect(page.getByRole("alert")).toContainText("Routing identity could not be checked");
});

test("a linked routed startup needing recovery is not offered for deletion", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "routed-recovery" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const savedRequest = page.locator(".flow-history");
  await expect(savedRequest).toContainText("recovery_required");
  await expect(savedRequest).toContainText("Run needs recovery");
  await expect(savedRequest.getByRole("button", { name: "Delete Fix the parser regression" })).toHaveCount(0);
});

test("a settled linked failure does not claim recovery is pending", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "routed-failed" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const savedRequest = page.locator(".flow-history");
  await expect(savedRequest).toContainText("Reviewed run retained; inspect status");
  await expect(savedRequest).not.toContainText("Run needs recovery");
  await expect(savedRequest.getByRole("button", { name: "Delete Fix the parser regression" })).toHaveCount(0);
});

test("an unresolved startup does not claim it is still running", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "routed-dispatching" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const savedRequest = page.locator(".flow-history");
  await expect(savedRequest).toContainText("Startup state unresolved; run may need recovery");
  await expect(savedRequest).not.toContainText("Run is starting");
  await expect(savedRequest.getByRole("button", { name: "Delete Fix the parser regression" })).toHaveCount(0);
});

test("a saved routed startup offers Stop after returning to Work", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-flow-history-v1": "routed-dispatching",
    "pytxo-preview-routed-stop-v1": "1",
  });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  await page.getByRole("button", { name: "Stop starting Fix the parser regression" }).click();
  await expect(page.getByRole("status").filter({ hasText: "Stop requested" })).toBeVisible();
});

test("a saved Shadow review shows local request and read-only proposed hosted disclosure", async ({ page }) => {
  const outbound: string[] = [];
  page.on("request", (request) => {
    if (/typesafe\.ai|\/v1\/routing\/evaluations|\/routing\/advice/i.test(request.url())) outbound.push(request.url());
  });
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "advisor-shadow" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const packet = page.getByRole("region", { name: "Routing packet for Reviewed routing task" });
  await expect(packet.getByRole("button", { name: "Inspect routing packet" })).toBeVisible();
  await expect(packet.getByLabel("Exact advisory request JSON")).toHaveCount(0);
  await packet.getByRole("button", { name: "Inspect routing packet" }).click();
  await expect(packet).toContainText("Local preview; viewing does not send or grant consent.");
  await expect(packet).toContainText("Recipient · pytxo-local-advisor-fixture/no-network/v1");
  await expect(packet.getByLabel("Exact advisory request JSON")).toContainText('"model":"browser-fixture-no-send"');
  await expect(packet).toContainText(/Exact request body · SHA-256 [a-f0-9]{64}/);
  await expect(packet).toContainText(/Packet SHA-256 [a-f0-9]{64}/);
  const proposed = packet.getByRole("region", { name: "Proposed hosted routing disclosure" });
  await expect(proposed).toContainText("This saved review covers the local fixture, not hosted routing.");
  await expect(proposed).toContainText("Proposed recipient · pytxo-hosted-routing/typesafe-systemone/v1");
  await expect(proposed).toContainText("Packet schema · 1 · initial_demand · execution_demand_v2");
  await expect(proposed).toContainText(/Hosted scope SHA-256 [a-f0-9]{64}/);
  const hostedPacket = JSON.parse(await proposed.getByLabel("Proposed hosted packet JSON").innerText());
  expect(hostedPacket).toEqual({ schema_version: 1, goal: "Classify reviewed repository task using coarse facts", features: ["task_kind_local_transformation", "reviewed_checks", "single_component_claimed", "context_claimed_complete", "role_unrestricted", "no_required_egress"], everyday_role: "everyday execution role", strong_role: "strong execution role" });
  await expect(proposed.getByRole("button", { name: /allow|send|connect/i })).toHaveCount(0);
  expect(await page.evaluate(() => localStorage.getItem("pytxo-preview-shadow-grant-state-v1"))).toBeNull();
  expect(outbound).toEqual([]);
  await proposed.scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("shadow-packet-hosted-proposal.png"), fullPage: true });
  await expect(packet.getByRole("button", { name: /consent|send|live/i })).toHaveCount(0);
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Tab");
  const close = packet.getByRole("button", { name: "Close packet" });
  await expect(close).toBeFocused();
  expect(await close.evaluate((button) => getComputedStyle(button).outlineStyle)).toBe("solid");
  await page.screenshot({ path: test.info().outputPath("shadow-packet-local-preview.png"), fullPage: true });
});

test("the inspected no-network Shadow grant can be enabled and revoked", async ({ page }) => {
  const outbound: string[] = [];
  page.on("request", (request) => {
    if (/typesafe\.ai|\/routing\/advice/i.test(request.url())) outbound.push(request.url());
  });
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "advisor-shadow" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const packet = page.getByRole("region", { name: "Routing packet for Reviewed routing task" });
  await packet.getByRole("button", { name: "Inspect routing packet" }).click();
  await expect(packet).toContainText("Workspace grant · Off · revision 0");
  await packet.getByRole("button", { name: "Allow local Shadow fixture" }).click();
  await expect(packet.getByRole("status")).toContainText("No request was sent");
  await expect(packet).toContainText("Workspace grant · Allowed for local Shadow fixture · revision 1");
  await packet.getByRole("button", { name: "Revoke local Shadow fixture" }).click();
  await expect(packet).toContainText("Workspace grant · Off · revision 2");
  await expect(packet.getByRole("button", { name: "Allow local Shadow fixture" })).toHaveCount(0);
  expect(outbound).toEqual([]);
});

test("a packet changed after inspection cannot grant local Shadow access", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "advisor-shadow" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const packet = page.getByRole("region", { name: "Routing packet for Reviewed routing task" });
  await packet.getByRole("button", { name: "Inspect routing packet" }).click();
  await page.evaluate(() => localStorage.setItem("pytxo-preview-flow-history-v1", "advisor-stale"));
  await packet.getByRole("button", { name: "Allow local Shadow fixture" }).click();
  await expect(packet.getByRole("alert")).toContainText("reviewed packet or workspace grant changed");
  await expect(packet.getByRole("button", { name: "Allow local Shadow fixture" })).toHaveCount(0);
});

test("an older workspace routing grant can still be revoked", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "advisor-shadow", "pytxo-preview-shadow-consent-v1": "other-scope" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const packet = page.getByRole("region", { name: "Routing packet for Reviewed routing task" });
  await packet.getByRole("button", { name: "Inspect routing packet" }).click();
  await expect(packet).toContainText("Older or different grant active");
  await packet.getByRole("button", { name: "Revoke existing routing grant" }).click();
  await expect(packet).toContainText("Workspace grant · Off · revision 2");
});

test("workspace routing access remains revocable after its saved review disappears", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/flow");
  await page.evaluate(() => localStorage.setItem("pytxo-preview-flow-history-v1", "advisor-shadow"));
  await page.reload();
  await page.locator(".flow-history > summary").click();
  const packet = page.getByRole("region", { name: "Routing packet for Reviewed routing task" });
  await packet.getByRole("button", { name: "Inspect routing packet" }).click();
  await packet.getByRole("button", { name: "Allow local Shadow fixture" }).click();
  await expect(page.getByRole("region", { name: "Experimental routing access" })).toContainText("local Shadow fixture");
  await page.screenshot({ path: test.info().outputPath("shadow-workspace-grant.png"), fullPage: true });
  await page.evaluate(() => localStorage.removeItem("pytxo-preview-flow-history-v1"));
  await page.reload();
  await expect(page.locator(".flow-history")).toHaveCount(0);
  await expect(page.getByRole("region", { name: "Routing packet for Reviewed routing task" })).toHaveCount(0);
  const workspaceAccess = page.getByRole("region", { name: "Experimental routing access" });
  await expect(workspaceAccess.getByRole("button", { name: "Revoke routing access" })).toBeVisible();
  await page.screenshot({ path: test.info().outputPath("shadow-workspace-revoke-without-review.png"), fullPage: true });
  await workspaceAccess.getByRole("button", { name: "Revoke routing access" }).click();
  await expect(workspaceAccess.getByRole("status")).toContainText("revoked");
  await expect(workspaceAccess.getByRole("button", { name: "Revoke routing access" })).toHaveCount(0);
});

test("an unreadable saved routing grant is visible as an access error", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "advisor-shadow" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const packet = page.getByRole("region", { name: "Routing packet for Reviewed routing task" });
  await packet.getByRole("button", { name: "Inspect routing packet" }).click();
  await packet.getByRole("button", { name: "Allow local Shadow fixture" }).click();
  await page.evaluate(() => localStorage.setItem("pytxo-preview-shadow-consent-error-v1", "1"));
  await page.reload();
  const workspaceAccess = page.getByRole("region", { name: "Experimental routing access" });
  await expect(workspaceAccess.getByRole("alert")).toContainText("saved workspace routing grant could not be checked");
});

test("Rules saved reviews do not offer a packet action", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "advisor-rules" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  await expect(page.getByRole("button", { name: "Inspect routing packet" })).toHaveCount(0);
});

test("a Shadow review that goes stale while opening fails closed", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "advisor-shadow" });
  await page.goto("/#/flow");
  await page.locator(".flow-history > summary").click();
  const action = page.getByRole("button", { name: "Inspect routing packet" });
  await expect(action).toBeVisible();
  await page.evaluate(() => localStorage.setItem("pytxo-preview-flow-history-v1", "advisor-stale"));
  await action.click();
  const packet = page.getByRole("region", { name: "Routing packet for Reviewed routing task" });
  await expect(packet.getByRole("alert")).toContainText("no longer has a current Shadow packet");
  await expect(packet.getByLabel("Exact advisory request JSON")).toHaveCount(0);
  await expect(packet.getByRole("button", { name: "Inspect routing packet" })).toHaveCount(0);
});

test("a dispatch error refreshes durable status before offering another run", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-ade-state-v1": "codex-only",
    "pytxo-preview-flow-dispatch-recovery-v1": "1",
  });
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Fix src/parser.rs and add a regression test");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  const runButton = page.getByRole("button", { name: "Run", exact: true });
  await expect(runButton).toBeEnabled();
  await runButton.click();
  await expect(page.getByText("Browser fixture: routed startup requires recovery")).toBeVisible();
  await expect(runButton).toBeDisabled();
  await page.locator(".flow-history > summary").click();
  await expect(page.locator(".flow-history")).toContainText("Run needs recovery");
});
