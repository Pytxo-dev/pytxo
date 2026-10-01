import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await completeOnboarding(page);
});

test("Setup keeps the expanded harness catalog compact and truthful", async ({ page }) => {
  await page.goto("/#/settings");
  await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();

  await expect(page.getByRole("heading", { name: "Agent harnesses", exact: true })).toBeVisible();
  await expect(page.getByText("Beta agents", { exact: true })).toBeVisible();
  await expect(page.locator(".agent-row:visible")).toHaveCount(5);
  const more = page.getByText("Additional agents", { exact: true });
  await expect(more).toBeVisible();
  await expect(page.getByText("Grok Build", { exact: true })).not.toBeVisible();
  await page.screenshot({ path: test.info().outputPath("harness-catalog-collapsed.png") });

  await more.click();
  await expect(page.getByText("Grok Build", { exact: true })).toBeVisible();
  await expect(page.getByText("Factory Droid", { exact: true })).toBeVisible();
  await expect(page.getByText("Qwen Code", { exact: true })).toBeVisible();
  await expect(page.locator(".agent-row")).toHaveCount(14);
  await expect(page.locator(".agent-row", { hasText: "Grok Build" })).toContainText("grok --no-auto-update -p");
  await expect(page.locator(".agent-row", { hasText: "Goose" })).toContainText("goose run --no-session -t");
  await expect(page.locator(".agent-row", { hasText: "Qwen Code" })).toContainText("write execution stays disabled");

  const catalogPanel = page.locator(".supported-catalog").locator("xpath=..");
  await page.getByText("Editor and cloud integrations", { exact: true }).click();
  const integrationPanel = page.locator(".integration-options");
  const permissionPanel = page.locator(".settings-group").filter({
    has: page.getByRole("heading", { name: "Default permissions", exact: true }),
  });
  const [catalogBox, integrationBox, permissionBox] = await Promise.all([
    catalogPanel.boundingBox(),
    integrationPanel.boundingBox(),
    permissionPanel.boundingBox(),
  ]);
  expect(catalogBox).not.toBeNull();
  expect(integrationBox).not.toBeNull();
  expect(permissionBox).not.toBeNull();
  expect(integrationBox!.height).toBeLessThanOrEqual(112);
  expect(integrationBox!.y - (catalogBox!.y + catalogBox!.height)).toBeGreaterThanOrEqual(10);
  expect(permissionBox!.y - (integrationBox!.y + integrationBox!.height)).toBeGreaterThanOrEqual(10);
  await integrationPanel.scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("integration-spacing-after.png") });

  const markedHarnesses = ["claude", "agy", "codex", "cursor", "opencode", "gemini", "copilot", "aider", "grok", "droid", "cline", "goose", "qwen", "kimi"];
  const assetPaths = [
    "/ade/anthropic.svg",
    "/ade/antigravity.png",
    "/ade/openai-on-dark.svg",
    "/ade/openai-on-light.svg",
    "/ade/cursor-on-dark.svg",
    "/ade/cursor-on-light.svg",
    "/ade/opencode.svg",
    "/ade/gemini.svg",
    "/ade/copilot.svg",
    "/ade/aider.png",
    "/ade/grok-on-dark.svg",
    "/ade/grok-on-light.svg",
    "/ade/factory-droid.svg",
    "/ade/cline.svg",
    "/ade/goose.svg",
    "/ade/qwen.svg",
    "/ade/kimi.svg",
  ];
  const assetStatuses = await page.evaluate(async (paths) =>
    Promise.all(paths.map(async (path) => ({ path, status: (await fetch(path)).status }))), assetPaths);
  expect(assetStatuses).toEqual(assetPaths.map((path) => ({ path, status: 200 })));
  const clineAsset = await page.evaluate(async () => (await fetch("/ade/cline.svg")).text());
  expect(clineAsset).toContain('viewBox="0 0 466.73 487.04"');
  expect(clineAsset).not.toContain("#863bff");
  await expect(page.locator(".ade-identity")).toHaveCount(markedHarnesses.length);
  await expect(page.locator('.ade-identity img[src$=".ico"]')).toHaveCount(0);
  for (const id of markedHarnesses) {
    const logo = page.locator(`.ade-identity[data-ade-id="${id}"]`);
    await expect(logo).toBeVisible();
    await expect(logo).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
  }
  await expect(page.locator('.ade-identity[data-ade-id="codex"] .ade-logo-on-dark')).toBeVisible();
  await expect(page.locator('.ade-identity[data-ade-id="codex"] .ade-logo-on-light')).toBeHidden();
  await expect(page.locator('.ade-identity[data-ade-id="cursor"] .ade-logo-on-dark')).toBeVisible();
  await expect(page.locator('.ade-identity[data-ade-id="cursor"] .ade-logo-on-light')).toBeHidden();
  await expect(page.locator('.ade-identity[data-ade-id="grok"] .ade-logo-on-dark')).toBeVisible();
  await expect(page.locator('.ade-identity[data-ade-id="grok"] .ade-logo-on-light')).toBeHidden();
  await expect(page.locator('.ade-identity[data-ade-id="cline"] .ade-mark')).toBeVisible();
  await expect(page.locator('.ade-identity[data-ade-id="cline"] img')).toHaveCount(0);
  await page.getByRole("heading", { name: "Agent harnesses", exact: true }).scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("harness-catalog-expanded-top.png") });
  await page.locator('.ade-identity[data-ade-id="aider"]').scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("harness-catalog-expanded-middle.png") });
  await page.locator('.ade-identity[data-ade-id="kimi"]').scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("harness-catalog-expanded-bottom.png") });

  await page.evaluate(() => localStorage.setItem("pytxo-deck-theme", "light"));
  await page.reload();
  await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
  await page.getByText("Additional agents", { exact: true }).click();
  await expect(page.locator('html[data-chroma-theme="light"]')).toBeVisible();
  await expect(page.locator('.ade-identity[data-ade-id="codex"] .ade-logo-on-dark')).toBeHidden();
  await expect(page.locator('.ade-identity[data-ade-id="codex"] .ade-logo-on-light')).toBeVisible();
  await expect(page.locator('.ade-identity[data-ade-id="cursor"] .ade-logo-on-dark')).toBeHidden();
  await expect(page.locator('.ade-identity[data-ade-id="cursor"] .ade-logo-on-light')).toBeVisible();
  await expect(page.locator('.ade-identity[data-ade-id="grok"] .ade-logo-on-dark')).toBeHidden();
  await expect(page.locator('.ade-identity[data-ade-id="grok"] .ade-logo-on-light')).toBeVisible();
  await page.getByRole("heading", { name: "Agent harnesses", exact: true }).scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("harness-catalog-expanded-light-top.png") });
  await page.locator('.ade-identity[data-ade-id="aider"]').scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("harness-catalog-expanded-light-middle.png") });
  await page.locator('.ade-identity[data-ade-id="kimi"]').scrollIntoViewIfNeeded();
  await page.screenshot({ path: test.info().outputPath("harness-catalog-expanded-light-bottom.png") });
});

test("vendor-managed harnesses remain selectable without a fake connected state", async ({ page }) => {
  await page.goto("/#/flow");
  await page.getByLabel("Agent CLI", { exact: true }).selectOption("gemini");

  await expect(page.getByText("Gemini CLI: available · vendor-managed authentication", { exact: true })).toBeVisible();
  await page.getByLabel("What should Pytxo do?").fill("Explain the selected harness contract");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await page.getByText("Permissions and technical details", { exact: true }).click();
  await expect(page.getByText(/gemini --skip-trust -p/)).toBeVisible();
});
