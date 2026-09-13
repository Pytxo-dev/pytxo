import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

for (const viewport of [{ width: 1440, height: 900 }, { width: 1024, height: 700 }, { width: 860, height: 560 }]) {
  test(`menus adapt without clipping controls at ${viewport.width}`, async ({ page }, info) => {
    await page.setViewportSize(viewport);
    await completeOnboarding(page);
    await page.goto("/#/setup");
    const picker = page.getByRole("combobox", { name: "Settings section", exact: true });
    if (await picker.isVisible()) await picker.selectOption("appearance");
    else await page.getByRole("button", { name: "Appearance", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Appearance", exact: true })).toBeInViewport();
    const fit = async () => expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    await fit();
    await page.screenshot({ path: info.outputPath("setup-appearance.png") });
    // Below the native minimum, exercise the compact picker too. At 860 the
    // primary sidebar already collapses, leaving room for the section rail.
    await page.setViewportSize({ width: 700, height: 560 });
    await expect(picker).toHaveValue("appearance");
    await picker.selectOption("general");
    await expect(page.getByRole("button", { name: "Reduced motion", exact: true })).toBeInViewport();
    await page.setViewportSize(viewport);
    await page.getByRole("button", { name: "Open command palette" }).click();
    const commands = page.getByRole("dialog", { name: "Command menu" });
    await expect(commands.locator("footer")).toBeInViewport();
    await page.keyboard.press("End");
    await expect(commands.locator('[aria-selected="true"]')).toBeInViewport();
    await expect(commands.locator("footer")).toBeInViewport();
    await page.screenshot({ path: info.outputPath("command-menu.png") });
    await page.keyboard.press("Escape");
    await page.getByTitle("Switch workspace", { exact: true }).click();
    const switcher = page.getByRole("dialog", { name: "Switch workspace", exact: true });
    await expect(switcher.getByRole("button", { name: "Workspace settings", exact: true })).toBeInViewport();
    await page.screenshot({ path: info.outputPath("workspace-switcher.png") });
    await switcher.getByRole("button", { name: "Workspace settings", exact: true }).click();
    const editor = page.getByRole("dialog", { name: /^Workspace settings/ });
    const close = editor.getByRole("button", { name: "Close", exact: true });
    const before = (await close.boundingBox())!;
    await editor.locator(".sheet-body").evaluate(el => { el.scrollTop = el.scrollHeight; });
    expect((await close.boundingBox())!.y).toBe(before.y);
    await expect(close).toBeInViewport();
    await page.screenshot({ path: info.outputPath("workspace-settings.png") });
    await page.keyboard.press("Escape");
    await page.goto("/#/approvals");
    const inbox = page.getByRole("dialog", { name: "Approvals inbox" });
    // Metadata must scroll with the detail, never collapse into a clipped card.
    expect(await inbox.locator("dl").evaluate(el => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(1);
    const actions = inbox.locator(".actions");
    const actionBox = (await actions.boundingBox())!;
    await inbox.locator(".detail-body").evaluate(el => { el.scrollTop = el.scrollHeight; });
    expect((await actions.boundingBox())!.y).toBe(actionBox.y);
    await expect(inbox.getByRole("button", { name: /Approve and apply/ })).toBeInViewport();
    await expect(inbox.getByRole("button", { name: "Close approvals" })).toBeInViewport();
    await fit();
    await page.screenshot({ path: info.outputPath("approvals.png") });
    await page.keyboard.press("Escape");
    await page.goto("/#/work");
    await page.locator(".layout-menu summary").click();
    await expect(page.getByRole("button", { name: "Save current layout", exact: true })).toBeInViewport();
    await page.screenshot({ path: info.outputPath("layout-menu.png") });
    await page.keyboard.press("Escape");
    await expect(page.locator(".layout-menu")).not.toHaveAttribute("open", "");
    await expect(page.locator(".layout-menu summary")).toBeFocused();
  });
}


test("an empty approvals inbox uses one compact message", async ({ page }, info) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 860, height: 560 });
  await page.goto("/#/approvals");
  const inbox = page.getByRole("dialog", { name: "Approvals inbox" });
  // Resolve only the deterministic browser fixture requests.
  await inbox.getByRole("button", { name: /Deny and discard/ }).click();
  await inbox.getByRole("button", { name: /Deny action/ }).click();
  await expect(inbox.getByText("Inbox clear", { exact: true }).and(page.locator(":visible"))).toHaveCount(1);
  expect((await inbox.boundingBox())!.height).toBeLessThanOrEqual(320);
  await expect(inbox.getByRole("button", { name: "Close approvals" })).toBeInViewport();
  await page.screenshot({ path: info.outputPath("empty-approvals.png") });
});
