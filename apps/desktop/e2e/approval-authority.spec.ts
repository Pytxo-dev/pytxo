import { expect, test } from "@playwright/test";
import { approvalPresentation } from "../src/lib/approval-presentation";
import type { HitlDto } from "../src/lib/types";
import { completeOnboarding } from "./helpers";

test("legacy flush decisions require candidate review and denial does not promise deletion", () => {
  for (const action of ["blast.flush", "Flush Blast Shield workspace"]) {
    const shown = approvalPresentation({ action } as HitlDto);
    expect(shown.requiresCandidateReview).toBe(true);
    expect(shown.consequence).toContain("Review");
    expect(shown.denyLabel).toBe("Deny request");
    expect(shown.deniedMessage).not.toMatch(/discard|delet/i);
  }
  expect(approvalPresentation({ action: "net.egress" } as HitlDto).requiresCandidateReview).toBe(false);
});

test("approval shortcut cannot authorize a legacy repository flush", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/approvals");
  const inbox = page.getByRole("dialog", { name: "Approvals inbox" });
  await expect(inbox.getByRole("heading", { name: "Review repository changes" })).toBeVisible();
  await page.keyboard.press("Control+Enter");
  await expect(inbox).toBeVisible();
  await expect(inbox.locator(".count")).toHaveText("2 open");
  await expect(inbox.getByRole("button", { name: /Approve and apply/ })).toHaveCount(0);
  await inbox.getByRole("button", { name: "Review changes", exact: true }).click();
  await expect(inbox).toBeHidden();
  await expect(page.locator("#run-review-title")).toBeVisible();
  await page.goto("/#/approvals");
  await expect(inbox.locator(".count")).toHaveText("2 open");
  await inbox.getByRole("button", { name: /Deny request/ }).click();
  await expect(inbox.getByRole("status")).toContainText("The workspace flush will not proceed.");
  await expect(inbox.getByRole("heading", { name: "Allow network access" })).toBeVisible();
  await page.keyboard.press("Control+Enter");
  await expect(inbox.getByText("Inbox clear").first()).toBeVisible();
});
