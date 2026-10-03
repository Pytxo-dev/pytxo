import { expect, test, type Page } from "@playwright/test";
import { completeOnboarding } from "./helpers";

// Headless Chromium hides scrollbars by default, disabling their drag targets.
test.use({ launchOptions: { ignoreDefaultArgs: ["--hide-scrollbars"] } });

async function openReview(page: Page) {
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await page.getByRole("button", { name: "Review prepared changes", exact: true }).click();
}

for (const theme of ["void", "light"]) {
  for (const width of [1280, 960]) {
    test(`readable chrome, tasks and Apply controls at ${width} in ${theme}`, async ({ page }, testInfo) => {
      await page.setViewportSize({ width, height: 800 });
      await completeOnboarding(page, {
        "pytxo-deck-theme": theme,
        "pytxo-accent": "teal",
        "pytxo-preview-native-agent-ids-v1": "1",
        "pytxo-preview-candidate-check-v1": "passed",
      });
      await page.goto("/#/work");
      await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", theme);
      await expect(page.getByTestId("execution-map").getByText("Recorded workers", { exact: true })).toBeVisible();
      const command = page.getByRole("button", { name: "Open command palette" });
      await expect(command).toBeInViewport();
      expect(await command.evaluate(e => e.scrollWidth - e.clientWidth)).toBeLessThanOrEqual(1);
      if (width === 960) {
        await expect(page.locator(".sidebar")).toHaveClass(/collapsed/);
        await expect(command.locator("span")).toHaveCount(0);
        await expect(command).toHaveAttribute("title", /Search or command/);
      } else {
        await expect(command.locator("span")).toHaveText("Search");
        expect(await command.locator("span").evaluate(e => e.scrollWidth - e.clientWidth)).toBeLessThanOrEqual(1);
      }
      for (const selector of [".task-node button", ".task-node strong", ".run-bar button"]) {
        const overflow = await page.locator(selector).evaluateAll(nodes => nodes.map(e => e.scrollWidth - e.clientWidth));
        expect(overflow.length).toBeGreaterThan(0);
        expect(Math.max(...overflow)).toBeLessThanOrEqual(1);
      }
      await page.getByTestId("execution-map").getByRole("button", { name: "List", exact: true }).click();
      const architect = page.locator(".ledger").getByRole("button", { name: /run-8f2c:architect/ });
      await expect(architect.locator(".agent")).toHaveText("Agent 1");
      await expect(architect.locator(".agent")).toHaveAttribute("title", "run-8f2c:architect");
      await expect(architect.getByText("Completed", { exact: true })).toBeInViewport();
      await page.screenshot({ path: testInfo.outputPath("work.png") });

      await openReview(page);
      const trigger = page.getByRole("button", { name: "Apply reviewed changes", exact: true });
      await trigger.click();
      const dialog = page.getByRole("dialog", { name: /^Apply these \d+ reviewed files?\?$/ });
      await expect(dialog).toBeVisible();
      await expect(dialog.getByRole("button", { name: "Cancel", exact: true })).toBeFocused();
      for (const button of await dialog.getByRole("button").all()) {
        const bounds = await button.boundingBox();
        expect(bounds!.height).toBeGreaterThanOrEqual(40);
        expect(bounds!.width).toBeGreaterThanOrEqual(40);
      }
      await expect(dialog.locator("code")).toHaveText("pkg-71ad-immutable");
      await page.screenshot({ path: testInfo.outputPath("apply-dialog.png") });
      await page.keyboard.press("Escape");
      await expect(dialog).toBeHidden();
      await expect(trigger).toBeFocused();
      expect((await trigger.boundingBox())!.height).toBeGreaterThanOrEqual(40);
      const discard = page.getByRole("button", { name: "Discard review", exact: true });
      expect((await discard.boundingBox())!.height).toBeGreaterThanOrEqual(40);
      await discard.click();
      const destructive = page.getByRole("button", { name: "Discard permanently", exact: true });
      const contrast = await destructive.evaluate(e => {
        const context = document.createElement("canvas").getContext("2d")!;
        const luminance = (color: string) => {
          context.fillStyle = color;
          context.fillRect(0, 0, 1, 1);
          const channels = [...context.getImageData(0, 0, 1, 1).data].slice(0, 3).map(value => {
            const s = value / 255;
            return s <= .04045 ? s / 12.92 : ((s + .055) / 1.055) ** 2.4;
          });
          return channels[0] * .2126 + channels[1] * .7152 + channels[2] * .0722;
        };
        const style = getComputedStyle(e);
        const a = luminance(style.color), b = luminance(style.backgroundColor);
        return (Math.max(a, b) + .05) / (Math.min(a, b) + .05);
      });
      expect(contrast).toBeGreaterThanOrEqual(4.5);
      await page.screenshot({ path: testInfo.outputPath("discard-dialog.png") });
      await page.keyboard.press("Escape");
      expect(await page.locator(".mission-content > .content").evaluate(e => e.scrollWidth - e.clientWidth)).toBeLessThanOrEqual(1);
    });
  }
}

test("custom scrollbars remain draggable and keyboard navigation keeps chrome fixed", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await completeOnboarding(page);
  await openReview(page);
  const content = page.locator(".review-scroll");
  const chrome = await page.locator(".app-bar").boundingBox();
  const metrics = await content.evaluate(e => {
    const box = e.getBoundingClientRect();
    return { x: box.x, y: box.y, width: box.width, height: box.height, clientHeight: e.clientHeight, scrollHeight: e.scrollHeight, thumbWidth: getComputedStyle(e, "::-webkit-scrollbar").width, scrollbarWidth: getComputedStyle(e).scrollbarWidth };
  });
  expect(metrics.scrollbarWidth).toBe("auto");
  expect(metrics.thumbWidth).toBe("10px");
  const maxScroll = metrics.scrollHeight - metrics.clientHeight;
  expect(maxScroll).toBeGreaterThan(0);
  const thumbHeight = metrics.clientHeight * metrics.clientHeight / metrics.scrollHeight;
  await page.mouse.move(metrics.x + metrics.width - 6, metrics.y + thumbHeight / 2);
  await page.mouse.down();
  await page.mouse.move(metrics.x + metrics.width - 6, metrics.y + thumbHeight / 2 + 200, { steps: 12 });
  await page.mouse.up();
  await expect.poll(() => content.evaluate(e => e.scrollTop)).toBeGreaterThanOrEqual(Math.max(1, Math.floor(maxScroll * .75)));
  expect(await page.locator(".app-bar").boundingBox()).toEqual(chrome);
  await page.screenshot({ path: testInfo.outputPath("scrollbar-drag.png") });
  await page.getByRole("link", { name: "Work", exact: true }).focus();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("region", { name: "Work", exact: true }).getByRole("heading", { level: 1 })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollTop)).toBe(0);
});

test("manual and system theme choices survive navigation and reload", async ({ page }) => {
  await completeOnboarding(page);
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/#/settings");
  await page.getByRole("button", { name: "Appearance", exact: true }).click();
  await page.getByRole("button", { name: "Light", exact: true }).click();
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", "light");
  expect(await page.evaluate(() => localStorage.getItem("pytxo-deck-theme"))).toBe("light");
  await page.getByRole("link", { name: "Setup", exact: true }).click();
  await page.getByRole("button", { name: "Appearance", exact: true }).click();
  await page.getByRole("button", { name: "Match system theme", exact: true }).click();
  await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", "void");
  // The persisted choice is deliberately opposite the OS at next startup.
  await page.evaluate(() => localStorage.setItem("pytxo-deck-theme", "light"));
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", "void");
  await page.emulateMedia({ colorScheme: "light" });
  await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", "light");
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", "light");
});

test("appearance selection, focus and switches work in light mode at larger scale", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 960, height: 800 });
  await completeOnboarding(page, { "pytxo-deck-theme": "light" });
  await page.goto("/#/settings");
  await expect(page.locator("html")).toHaveAttribute("data-chroma-theme", "light");
  await page.getByRole("button", { name: "Appearance", exact: true }).click();
  const scale = page.getByRole("button", { name: "110%", exact: true });
  await scale.focus();
  await page.keyboard.press("Enter");
  await expect(scale).toHaveAttribute("aria-pressed", "true");
  const neutral = page.getByRole("button", { name: "Neutral", exact: true });
  await neutral.focus();
  await expect(neutral).toBeFocused();
  await expect(neutral).toHaveCSS("outline-style", "solid");
  const colors = await neutral.evaluate(e => ({ outline: getComputedStyle(e).outlineColor, foreground: getComputedStyle(document.documentElement).getPropertyValue("--pytxo-text-strong").trim() }));
  expect(colors.outline).toBe("rgb(15, 20, 25)");
  const matchSystem = page.getByRole("button", { name: "Match system theme", exact: true });
  await matchSystem.hover();
  await expect(matchSystem).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
  await matchSystem.click();
  await expect(matchSystem).toHaveAttribute("aria-pressed", "true");
  await expect(matchSystem).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
  await matchSystem.click();
  await page.getByRole("button", { name: "Light", exact: true }).click();
  const selected = await scale.evaluate(e => ({ foreground: getComputedStyle(e).color, background: getComputedStyle(e).backgroundColor }));
  expect(selected.foreground).not.toBe(selected.background);
  expect(await page.locator(".content").evaluate(e => e.scrollWidth - e.clientWidth)).toBeLessThanOrEqual(1);
  await page.screenshot({ path: testInfo.outputPath("appearance-light-110.png") });
});
