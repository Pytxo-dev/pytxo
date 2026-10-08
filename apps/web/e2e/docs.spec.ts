import { expect, test } from "@playwright/test";

test("docs home and getting-started pages load", async ({ page }) => {
  await page.goto("/docs");
  await expect(page).toHaveURL(/\/docs\/?$/);
  await expect(page.getByRole("heading", { level: 1, name: "Introduction" })).toBeVisible();

  await page.goto("/docs/getting-started/install");
  await expect(page.getByRole("heading", { level: 1, name: "Install" })).toBeVisible();
  await expect(page.getByText("Windows (PowerShell)", { exact: false })).toBeVisible();

  await page.goto("/docs/getting-started/review-and-apply");
  await expect(page.getByRole("heading", { level: 1, name: "Review and Apply" })).toBeVisible();
  await expect(page.getByText(/no pytxo apply/i)).toBeVisible();

  await page.goto("/docs/getting-started/first-mission");
  await expect(page.getByRole("heading", { level: 1, name: /First mission/i })).toBeVisible();
});

test("docs search indexes pytxo.toml", async ({ request, page }) => {
  const api = await request.get("/api/search?query=pytxo.toml");
  if (api.ok()) {
    const body = await api.text();
    expect(body.toLowerCase()).toContain("pytxo.toml");
    return;
  }

  await page.goto("/docs");
  const search = page.getByRole("button", { name: /search/i }).first();
  await search.click();
  const input = page.getByPlaceholder(/search/i).first();
  await input.fill("pytxo.toml");
  await expect(page.getByText(/pytxo\.toml/i).first()).toBeVisible();
});

test("docs compare and changelog are reachable", async ({ page }) => {
  await page.goto("/docs/compare");
  await expect(page.getByRole("heading", { level: 1, name: "Compare" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Conductor", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "Factory", exact: true })).toBeVisible();
  await expect(page.getByRole("link", { name: "BridgeMind", exact: true })).toBeVisible();
  await page.goto("/docs/compare/conductor");
  await expect(page.getByRole("heading", { level: 1, name: "Pytxo and Conductor" })).toBeVisible();
  await page.goto("/docs/reference/changelog");
  await expect(page.getByRole("heading", { level: 1, name: "Changelog" })).toBeVisible();
  await expect(page.getByText(/latest public GitHub and npm release/)).toBeVisible();
});

test("desktop setup docs follow the wizard before Add workspace", async ({ page }) => {
  await page.goto("/docs/getting-started/desktop-setup");
  await expect(page.getByRole("heading", { level: 1, name: "Desktop setup" })).toBeVisible();
  const body = page.locator("article, main").first();
  await expect(body).toContainText("Welcome");
  await expect(body).toContainText("Get started");
  await expect(body).toContainText("Select folder");
  await expect(body).toContainText("Enter Pytxo Desktop");
  await expect(body).toContainText("Add workspace");
  const text = await body.innerText();
  expect(text.indexOf("Get started")).toBeLessThan(text.indexOf("Add workspace"));
});

test("Desktop docs describe Work History and Setup, not the retired destinations", async ({ page }) => {
  await page.goto("/docs/concepts/desktop");
  await expect(page.getByRole("heading", { level: 1, name: "Pytxo Desktop" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Three destinations" })).toBeVisible();
  await expect(page.getByText("Work", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("History", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("Setup", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("Ops", { exact: true })).toHaveCount(0);
  await expect(page.getByText("New mission", { exact: true })).toHaveCount(0);
});
