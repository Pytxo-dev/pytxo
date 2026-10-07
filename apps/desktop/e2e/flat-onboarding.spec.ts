import { expect, test } from "@playwright/test";
import { clearOnboarding } from "./helpers";

for (const viewport of [{ width: 1280, height: 800 }, { width: 860, height: 560 }, { width: 640, height: 400 }, { width: 640, height: 353 }]) {
  test(`flat onboarding keeps actions reachable at ${viewport.width}x${viewport.height}`, async ({ page }, info) => {
    await page.setViewportSize(viewport);
    await clearOnboarding(page);
    await page.goto("/");
    await expect(page.locator(".setup-shell img")).toHaveCount(0);
    await expect(page.locator('[aria-current="step"]')).toContainText("Welcome");
    await expect(page.getByRole("heading", { level: 1 })).toBeFocused();
    const assertFrame = async () => {
      expect(await page.evaluate(() => document.documentElement.scrollHeight - innerHeight)).toBeLessThanOrEqual(1);
      const footer = (await page.locator(".setup-step__actions").boundingBox())!;
      expect(footer.y + footer.height).toBeLessThanOrEqual(viewport.height);
      if (viewport.width === 1280) expect(await page.locator(".setup-step__content").evaluate(el => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(1);
    };
    await assertFrame();
    await page.screenshot({ path: info.outputPath("onboarding-welcome.png") });
    await page.getByRole("button", { name: "Get started" }).click();
    await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
    await expect(page.locator('[aria-current="step"]')).toContainText("Agents");
    await assertFrame();
    await page.getByRole("button", { name: "Terminal tools" }).click();
    await expect(page.getByRole("heading", { name: "Terminal tools are optional" })).toBeVisible();
    await page.getByRole("button", { name: "Back", exact: true }).click();
    await page.getByRole("button", { name: /^Continue/ }).click();
    await expect(page.locator('[aria-current="step"]')).toContainText("Project");
    await assertFrame();
    await page.getByRole("button", { name: "Try the guided example" }).click();
    await expect(page.locator(".chosen")).toContainText("approval-risk-demo");
    await assertFrame();
    await page.screenshot({ path: info.outputPath("onboarding-ready.png") });
    await page.getByRole("button", { name: /Enter Pytxo Desktop/ }).click();
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
  });
}

test("recent workspace list scrolls independently of setup headings and actions", async ({ page }, info) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await clearOnboarding(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Get started" }).click();
  await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
  await page.evaluate(() => {
    Object.assign(window, { __TAURI_INTERNALS__: { invoke: async (cmd: string) => {
      if (cmd === "list_projects") return [];
      if (cmd === "list_domains_status") return Array.from({ length: 5 }, (_, i) => ({ domain_id: `C:/projects/long-project-${i}`, repo_root: `C:/projects/long-project-${i}`, is_available: true, is_temporary: false, project_id: null }));
      return [];
    } } });
  });
  await page.getByRole("button", { name: /^Continue/ }).click();
  await expect(page.locator(".recent__item")).toHaveCount(5);
  await page.waitForFunction(() => document.getAnimations().every(a => a.playState !== "running"));
  expect(await page.locator(".setup-step__content").evaluate(el => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(1);
  const heading = (await page.getByRole("heading", { name: "Where should they work?" }).boundingBox())!;
  await page.locator(".recent__list").evaluate(el => { el.scrollTop = el.scrollHeight; });
  expect((await page.getByRole("heading", { name: "Where should they work?" }).boundingBox())!.y).toBe(heading.y);
  await expect(page.getByRole("button", { name: "Select folder", exact: true })).toBeInViewport();
  await expect(page.getByRole("button", { name: "Skip for now", exact: true })).toBeInViewport();
  await page.screenshot({ path: info.outputPath("onboarding-workspace-list.png") });
});
