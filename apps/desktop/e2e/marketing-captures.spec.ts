import { expect, test, type Page } from "@playwright/test";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";

import { completeOnboarding } from "./helpers";

const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));
const DESKTOP_ROOT = fileURLToPath(new URL("../", import.meta.url));
const CAPTURE_DIR = path.join(DESKTOP_ROOT, "captures", "desktop-2");
const DOCS_CAPTURE_DIR = path.join(REPO_ROOT, "docs", "_attachments", "desktop-2");
const WEB_CAPTURE_DIR = path.join(REPO_ROOT, "apps", "web", "public", "product");
const DEMO_CAPTURE_DIR = path.join(DESKTOP_ROOT, "..", "demo-video", "public", "product");

const VIEWPORTS = [
  { slug: "1600x1000", width: 1600, height: 1000 },
  { slug: "1280x800", width: 1280, height: 800 },
  { slug: "960x640", width: 960, height: 640 },
] as const;

const ROUTES = [
  { route: "operations", heading: /^Ops/, marketing: true },
  { route: "workspaces", heading: "Workspaces", marketing: false },
  { route: "flow", heading: "New mission", marketing: true },
  { route: "approvals", heading: "Approvals", marketing: true },
  { route: "integrations", heading: "Agents", marketing: true },
  { route: "settings", heading: "Appearance", marketing: false },
  { route: "topology-focus", heading: "Mission", marketing: false },
  { route: "run-review", heading: "Run Review", marketing: true },
] as const;

const RETRYABLE_WRITE_CODES = new Set(["EACCES", "EBUSY", "EPERM", "UNKNOWN"]);

async function writeCapture(filePath: string, png: Buffer) {
  try {
    const current = await readFile(filePath);
    if (current.equals(png)) return;
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== "ENOENT") throw error;
  }

  for (let attempt = 1; attempt <= 5; attempt += 1) {
    try {
      await writeFile(filePath, png);
      return;
    } catch (error) {
      const code = (error as NodeJS.ErrnoException).code;
      if (!code || !RETRYABLE_WRITE_CODES.has(code) || attempt === 5) {
        throw error;
      }
      await delay(attempt * 100);
    }
  }
}

async function prepareRoute(page: Page, route: (typeof ROUTES)[number]) {
  await completeOnboarding(
    page,
    route.route === "run-review"
      ? { "pytxo-preview-review-state-v1": "ready" }
      : undefined,
  );
  await page.clock.install({ time: new Date("2026-01-15T10:00:00.000Z") });
  await page.goto(`/#/${route.route}`);
  await expect(page.getByRole("heading", { name: route.heading, exact: true })).toBeVisible();

  if (route.route === "flow") {
    await page
      .getByLabel("Mission outcome")
      .fill("Ship the approval workflow with isolated changes and verification");
    await page.getByRole("button", { name: "Build plan", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Review plan" })).toBeVisible();
  }

  if (route.route === "approvals") {
    await expect(
      page.getByRole("heading", { name: "Apply reviewed workspace changes" }),
    ).toBeVisible();
  }

  await page.evaluate(() => document.fonts.ready);
}

test.describe("@marketing-capture current Desktop product captures", () => {
  test.describe.configure({ mode: "serial" });

  test.beforeAll(async () => {
    await mkdir(CAPTURE_DIR, { recursive: true });
    await mkdir(DOCS_CAPTURE_DIR, { recursive: true });
    await mkdir(WEB_CAPTURE_DIR, { recursive: true });
    await mkdir(DEMO_CAPTURE_DIR, { recursive: true });
  });

  for (const viewport of VIEWPORTS) {
    for (const route of ROUTES) {
      test(`${route.route} is truthful at ${viewport.slug}`, async ({ page }) => {
        await page.setViewportSize({ width: viewport.width, height: viewport.height });
        await page.emulateMedia({ reducedMotion: "reduce" });
        await prepareRoute(page, route);

        const png = await page.screenshot({
          animations: "disabled",
          caret: "hide",
          fullPage: false,
          scale: "css",
        });

        expect(png.byteLength).toBeGreaterThan(40_000);
        await writeCapture(path.join(CAPTURE_DIR, `${route.route}-${viewport.slug}.png`), png);
        await writeCapture(path.join(DOCS_CAPTURE_DIR, `${route.route}-${viewport.slug}.png`), png);

        if (
          route.marketing &&
          (viewport.slug === "1600x1000" || viewport.slug === "960x640")
        ) {
          await writeCapture(
            path.join(WEB_CAPTURE_DIR, `${route.route}-${viewport.slug}.png`),
            png,
          );
        }

        if (route.marketing && viewport.slug === "1600x1000") {
          await writeCapture(
            path.join(DEMO_CAPTURE_DIR, `${route.route}-${viewport.slug}.png`),
            png,
          );
        }

        if (route.route === "run-review") {
          await page.getByRole("button", { name: "Apply reviewed changes" }).click();
          await expect(page.getByText("Applied successfully", { exact: true })).toBeVisible();
          const appliedPng = await page.screenshot({
            animations: "disabled",
            caret: "hide",
            fullPage: false,
            scale: "css",
          });
          await writeCapture(path.join(CAPTURE_DIR, `run-applied-${viewport.slug}.png`), appliedPng);
          await writeCapture(path.join(DOCS_CAPTURE_DIR, `run-applied-${viewport.slug}.png`), appliedPng);
          if (viewport.slug === "1600x1000" || viewport.slug === "960x640") {
            await writeCapture(
              path.join(WEB_CAPTURE_DIR, `run-applied-${viewport.slug}.png`),
              appliedPng,
            );
          }
        }

      });
    }
  }
});
