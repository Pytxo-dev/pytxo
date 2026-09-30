import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("explicit Claude subscription experiment keeps its reviewed prompt fixed and does not become the default", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-ade-state-v1": "codex-only",
    "pytxo-preview-claude-route-v1": "1",
  });
  await page.goto("/#/flow");
  await expect(page.getByLabel("Agent CLI")).toHaveValue("codex");
  await page.getByLabel("Agent CLI").selectOption("claude-proposal-route-experiment");
  await expect(page.getByText("Claude proposal route: configured · account and model readiness pending Run")).toBeVisible();
  await page.getByLabel("What should Pytxo do?").fill("Update src/api.ts with the reviewed behavior");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.locator(".plan-wave")).toHaveCount(1);
  await expect(page.getByLabel("Task claude-proposal prompt")).toHaveAttribute("readonly", "");
  await expect(page.getByText(/may consume account quota even if no candidate is produced/)).toBeVisible();
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();
  await page.reload();
  await expect(page.getByLabel("Agent CLI")).toHaveValue("codex");
});

test("reviewed Claude plan discloses the separate two-call repair limit", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-ade-state-v1": "codex-only",
    "pytxo-preview-claude-route-v1": "1",
    "pytxo-preview-claude-repair-v1": "1",
  });
  await page.goto("/#/flow");
  await page.getByLabel("Agent CLI").selectOption("claude-proposal-route-experiment");
  await page.getByLabel("What should Pytxo do?").fill("Update src/api.ts with the reviewed behavior");
  await page.getByLabel("Reviewed task type").selectOption("local_transformation");
  await page.getByLabel("Task context is complete").check();
  await page.getByLabel("Cross-component requirement").selectOption("no");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByText(/up to two Claude subscription worker calls: Haiku first/)).toBeVisible();
  await page.getByLabel("Reviewed task type").selectOption("architecture");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByText(/allows one Claude subscription worker call/)).toBeVisible();
});

test("starting a reviewed routed run exposes Stop while dispatch is pending", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-ade-state-v1": "codex-only",
    "pytxo-preview-claude-route-v1": "1",
    "pytxo-preview-routed-stop-v1": "1",
  });
  await page.goto("/#/flow");
  await page.getByLabel("Agent CLI").selectOption("claude-proposal-route-experiment");
  await page.getByLabel("What should Pytxo do?").fill("Update src/api.ts with the reviewed behavior");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await page.getByRole("button", { name: "Run", exact: true }).click();
  await page.getByRole("button", { name: "Stop starting run" }).click();
  await expect(page.getByRole("status").filter({ hasText: "Stop requested" })).toBeVisible();
  await expect(page.getByText("Stop was requested. Check the saved request for the final state before trying again.")).toBeVisible();
});

test("experimental task facts are explicit and changing them invalidates the review", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-ade-state-v1": "codex-only",
    "pytxo-preview-claude-route-v1": "1",
  });
  await page.goto("/#/flow");
  await expect(page.getByLabel("Reviewed task type")).toHaveCount(0);
  await page.getByLabel("Agent CLI").selectOption("claude-proposal-route-experiment");
  await expect(page.getByLabel("Reviewed task type")).toHaveValue("other");
  await expect(page.getByLabel("Task context is complete")).not.toBeChecked();
  await expect(page.getByLabel("Cross-component requirement")).toHaveValue("unknown");
  await page.getByLabel("What should Pytxo do?").fill("Diagnose src/api.ts and fix the recorded failure");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();
  await page.getByLabel("Reviewed task type").selectOption("diagnosis");
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
  await expect(page.getByLabel("Repeatable symptom supplied")).toHaveValue("unknown");
  await expect(page.getByLabel("Specific cause hypothesis supplied")).toHaveValue("unknown");
  await page.getByLabel("Task context is complete").check();
  await page.getByLabel("Cross-component requirement").selectOption("no");
  await page.getByLabel("Repeatable symptom supplied").selectOption("yes");
  await page.getByLabel("Specific cause hypothesis supplied").selectOption("no");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();
  await page.getByLabel("Specific cause hypothesis supplied").selectOption("yes");
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
  await page.getByLabel("What should Pytxo do?").fill("Diagnose src/api.ts with a different acceptance condition");
  await expect(page.getByLabel("Reviewed task type")).toHaveValue("other");
  await expect(page.getByLabel("Task context is complete")).not.toBeChecked();
  await expect(page.getByLabel("Cross-component requirement")).toHaveValue("unknown");
  await page.getByLabel("Reviewed task type").selectOption("diagnosis");
  await expect(page.getByLabel("Repeatable symptom supplied")).toHaveValue("unknown");
  await expect(page.getByLabel("Specific cause hypothesis supplied")).toHaveValue("unknown");
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
});

test("reusing a different request clears prior experimental routing claims", async ({ page }) => {
  await completeOnboarding(page, {
    "pytxo-preview-ade-state-v1": "codex-only",
    "pytxo-preview-claude-route-v1": "1",
    "pytxo-preview-flow-history-v1": "reuse",
  });
  await page.goto("/#/flow");
  await page.getByLabel("Agent CLI").selectOption("claude-proposal-route-experiment");
  await page.getByLabel("Reviewed task type").selectOption("documentation");
  await page.getByLabel("Task context is complete").check();
  await page.getByLabel("Cross-component requirement").selectOption("no");
  await page.getByText("Saved requests").click();
  await page.getByRole("button", { name: /Use as new request: Fix the parser regression/ }).click();
  await page.getByLabel("Agent CLI").selectOption("claude-proposal-route-experiment");
  await expect(page.getByLabel("Reviewed task type")).toHaveValue("other");
  await expect(page.getByLabel("Task context is complete")).not.toBeChecked();
  await expect(page.getByLabel("Cross-component requirement")).toHaveValue("unknown");
});
