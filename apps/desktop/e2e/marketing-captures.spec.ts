import { expect, test, type Page } from "@playwright/test";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";

import { completeOnboarding } from "./helpers";
import { MARKETING_ROUTES, MARKETING_VIEWPORTS } from "../../web/scripts/product-asset-manifest.mjs";

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

/**
 * Capture slugs follow the canonical routes so a marketing asset can never show
 * a destination the product no longer has. Legacy slugs are gone rather than
 * aliased: a stale filename is the mechanism by which an old screenshot
 * survives a redesign.
 */
const ROUTES = [
  { route: "work", heading: "Work", marketing: true },
  { route: "fleet", heading: "Work", marketing: true },
  { route: "history", heading: "History", marketing: true },
  { route: "flow", heading: "New work", marketing: true },
  { route: "approvals", heading: "Approvals", marketing: true },
  { route: "setup", heading: "Appearance", marketing: false },
  { route: "integrations", heading: "Agents & permissions", marketing: true },
  { route: "workspaces", heading: "Workspaces", marketing: false },
  { route: "run-review", heading: "Review changes", marketing: true },
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
      ? {
          "pytxo-preview-review-state-v1": "ready",
          "pytxo-preview-candidate-check-v1": "passed",
          "pytxo-preview-flow-history-v1": "ready",
        }
      : route.route === "fleet"
        ? { "pytxo-preview-fleet-v1": "1" }
        : route.route === "flow"
          ? { "pytxo-preview-split-delay-v1": "50" }
          : undefined,
  );
  await page.clock.install({ time: new Date("2026-01-15T10:00:00.000Z") });
  // The fleet capture is the Work view of a mixed-CLI run.
  await page.goto(`/#/${route.route === "fleet" ? "work" : route.route}`);
  if (route.route === "fleet") await expect(page.getByTestId("fleet-board").getByRole("log", { name: "Recent output from Claude Code" })).toContainText("Update(");
  else if (route.route === "work") await expect(page.getByRole("region", { name: "Work", exact: true }).getByRole("heading", { level: 1 })).toBeVisible();
  else await expect(route.route === "run-review" ? page.locator("#run-review-title") : page.getByRole("heading", { name: route.heading, exact: true })).toBeVisible();

  if (route.route === "flow") {
    // One plain request, split by an agent into owned task lines, then planned.
    await page
      .getByLabel("What should Pytxo do?")
      .fill("Add search and status filtering with tests, a dark theme that follows the system, and a Spanish translation with a language switch.");
    await page.getByRole("button", { name: "Split with OpenAI Codex" }).click();
    await expect(page.getByText("OpenAI Codex proposed 4 tasks")).toBeVisible();
    await page.getByRole("button", { name: "Build plan", exact: true }).click();
    await expect(page.getByRole("heading", { name: "Review plan" })).toBeVisible();
  }

  if (route.route === "approvals") {
    await expect(
      page.getByRole("heading", { name: "Review repository changes" }),
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
        // Screen entry moves focus for keyboard users; a still has no keyboard, so drop the ring.
        await page.evaluate(() => (document.activeElement as HTMLElement | null)?.blur());

        const png = await page.screenshot({
          animations: "disabled",
          caret: "hide",
          fullPage: false,
          scale: "css",
        });

        // A shell with an empty screen encodes to about 17 KB at 960x640; the
        // sparsest real screen (History's run list in mono) is about 39 KB.
        expect(png.byteLength).toBeGreaterThan(28_000);
        await writeCapture(path.join(CAPTURE_DIR, `${route.route}-${viewport.slug}.png`), png);
        await writeCapture(path.join(DOCS_CAPTURE_DIR, `${route.route}-${viewport.slug}.png`), png);

        if (
          MARKETING_ROUTES.includes(route.route) &&
          MARKETING_VIEWPORTS.some((size: { slug: string }) => size.slug === viewport.slug)
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
          await page.getByRole("button", { name: "Apply exact package" }).click();
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

  test("work is truthful at 1920x1080 for the demo still", async ({ page }) => {
    const route = ROUTES.find((item) => item.route === "work");
    if (!route) throw new Error("expected a Work capture route");
    await page.setViewportSize({ width: 1920, height: 1080 });
    await page.emulateMedia({ reducedMotion: "reduce" });
    await prepareRoute(page, route);

    const png = await page.screenshot({
      animations: "disabled",
      caret: "hide",
      fullPage: false,
      scale: "css",
    });

    expect(png.byteLength).toBeGreaterThan(40_000);
    await writeCapture(path.join(DEMO_CAPTURE_DIR, "work-1920x1080.png"), png);
  });
});
