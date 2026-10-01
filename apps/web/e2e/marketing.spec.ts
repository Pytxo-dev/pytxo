import { expect, test, type Page } from "@playwright/test";

async function expectNoHorizontalOverflow(page: Page) {
  const overflow = await page.evaluate(() => {
    const main = document.querySelector("main");
    return {
      document: document.documentElement.scrollWidth - document.documentElement.clientWidth,
      main: main ? main.scrollWidth - main.clientWidth : null,
    };
  });
  expect(overflow.document).toBeLessThanOrEqual(1);
  expect(overflow.main).toBeLessThanOrEqual(1);
}

async function renderedLineCount(locator: ReturnType<Page["getByRole"]>) {
  return locator.evaluate((element) => {
    const style = getComputedStyle(element);
    const lineHeight = Number.parseFloat(style.lineHeight);
    return element.getBoundingClientRect().height / lineHeight;
  });
}

for (const width of [1440, 390]) {
  test(`launch journey is scoped and readable at ${width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 900 });
    await page.emulateMedia({ reducedMotion: "reduce" });
    await page.goto("/");
    const hero = page.getByTestId("marketing-hero");
    await expect(hero.getByRole("heading", { level: 1 })).toContainText("Agents do the work.");
    await expect(hero.getByRole("heading", { level: 1 })).toContainText("You decide what lands.");
    await expect(hero.getByText(/Give your coding agent a task/)).toBeVisible();
    await expect(hero.getByRole("link", { name: "Download current v1.2.1" })).toBeInViewport();
    await expect(hero.getByText("The walkthrough below previews the unpublished v1.2.2 Desktop interface.")).toBeVisible();
    await expect(hero.getByRole("link", { name: "First mission" })).toHaveAttribute("href", "/docs/getting-started/first-mission");
    await expect(page.getByRole("tab", { name: "Review & Apply" })).toHaveAttribute("aria-selected", "true");
    const walkthrough = page.getByTestId("product-walkthrough");
    await expect(walkthrough.getByText("Unpublished v1.2.2 · browser fixture")).toBeVisible();
    const panel = walkthrough.getByRole("tabpanel");
    if (width < 1280) {
      const details = panel.getByTestId("mobile-product-details");
      await expect(details.getByRole("img")).toHaveCount(2);
      await expect(details.getByRole("img").first()).toHaveJSProperty("naturalWidth", 1600);
      await expect(details).toContainText("Checks belong to this candidate");
      await expect(details).toContainText("Apply is an explicit decision");
    } else {
      await expect(panel.getByRole("img")).toHaveJSProperty("naturalWidth", 1600);
      expect((await panel.getByRole("img").boundingBox())!.width).toBeGreaterThan(900);
    }
    await expect(page.getByText("Interface previews using browser fixtures, not a recorded mission or proof of execution.")).toBeVisible();
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`launch-home-${width}.png`), fullPage: true });
    await hero.getByRole("link", { name: "Download current v1.2.1" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "Download" })).toBeVisible();
    await expect(page.getByText(/Workspace .* is unpublished/)).toBeVisible();
    await expect(page.getByText("The download below installs the current public v1.2.1 build.")).toBeVisible();
    await expect(page.getByText("The product source repository is currently private.")).toBeVisible();
    await expect(page.getByRole("button", { name: "Not yet" })).toHaveCount(2);
    await expect(page.getByText("The CLI does not Apply repository changes")).toBeVisible();
    await expect(page.getByText("planning, waves, isolation, and Apply")).toHaveCount(0);
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`launch-download-${width}.png`), fullPage: true });
  });
}

test("plans present current availability without a public subscription checkout", async ({ page }) => {
  await page.goto("/plans");
  await expect(page.getByRole("heading", { level: 1, name: "Plans and availability" })).toBeVisible();
  await expect(page.getByText("Core works today on your machine without checkout.", { exact: false })).toBeVisible();
  await expect(page.getByRole("link", { name: /Subscribe to/i })).toHaveCount(0);
  await expect(page.locator("main").getByRole("link", { name: "Sign in" })).toHaveCount(0);
});

test("install commands can be copied intact on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.context().grantPermissions(["clipboard-read", "clipboard-write"], { origin: process.env.PLAYWRIGHT_BASE_URL || "http://127.0.0.1:3101" });
  await page.goto("/download");
  const copyButton = page.getByRole("button", { name: "Copy Windows PowerShell install command" });
  const command = await copyButton.locator("..").locator("..").locator("pre").textContent();
  expect(command).toMatch(/^irm https:\/\/raw\.githubusercontent\.com\/.+\/install\.ps1 \| iex$/);
  await copyButton.click();
  await expect(copyButton).toHaveText("Copied");
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(command);
  await expectNoHorizontalOverflow(page);
});

test("walkthrough supports keyboard navigation without simulated live state", async ({ page }) => {
  await page.goto("/");
  const review = page.getByRole("tab", { name: "Review & Apply" });
  await review.focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("tab", { name: "Recorded outcome" })).toBeFocused();
  await expect(page.getByRole("tabpanel")).toContainText("A saved history entry is not itself proof");
  await page.keyboard.press("Home");
  await expect(page.getByRole("tab", { name: "Execution" })).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("tabpanel")).toContainText(
    "Follow recorded workers on the canvas, inspect their evidence, and open output when needed",
  );
});

test("homepage explains workflow, limits and supported Beta in order", async ({ page }) => {
  await page.goto("/");
  const ids = ["marketing-hero", "boundary-section", "compatibility-section", "get-it-section"];
  const tops = await Promise.all(ids.map(id => page.getByTestId(id).evaluate(el => el.getBoundingClientRect().top)));
  expect([...tops].sort((a,b)=>a-b)).toEqual(tops);
  await expect(page.getByTestId("product-walkthrough").getByRole("tab")).toHaveCount(3);
  await expect(page.getByTestId("boundary-section")).toContainText("does not control every host or network side effect");
  await expect(page.getByTestId("compatibility-section")).toContainText("Codex CLI installed and authenticated");
  await expect(page.getByTestId("evidence-section")).toHaveCount(0);
  await expect(page.locator(".aperture-marquee")).toHaveCount(0);
});

test("the evidence page states the corpus, caveats, and source access limit", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/evidence");

  await expect(page.getByRole("heading", { level: 1, name: "Evidence and its limits." })).toBeVisible();
  await expect(page.getByRole("heading", { name: "What it does not claim" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Caveats that weaken the number" })).toBeVisible();
  await expect(page.getByText(/worktree was dirty at capture time/)).toBeVisible();
  await expect(page.getByText(/The source link requires repository access/)).toBeVisible();
  await expect(
    page.getByRole("link", { name: "tooling/benchmarks/results/signal-real-repo.json" }).first(),
  ).toBeVisible();
  await expectNoHorizontalOverflow(page);
});

for (const width of [1440, 390]) {
  test(`native evidence links resolve to the displayed artifact at ${width}px`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto("/evidence");
    const nativeLink = page.getByRole("link", { name: "Native run record", exact: true });
    const href = await nativeLink.getAttribute("href");
    expect(href).toBeTruthy();
    const response = await page.request.get(href!);
    expect(response.ok()).toBe(true);
    expect(response.headers()["content-type"]).toContain("application/json");
    const record = await response.json();
    expect(record.run_status).toBe("completed");
    expect(record.native_apply.applied_files_match_frozen_digests).toBe(true);
    expect(record.msi_sha256).toMatch(/^[a-f0-9]{64}$/);
    await expect(page.getByText(`Recorded MSI SHA256: ${record.msi_sha256}`, { exact: true })).toBeVisible();
    const currentHref = await page.getByRole("link", { name: "September 8 checkpoint record", exact: true }).getAttribute("href");
    expect(currentHref).toBeTruthy();
    const currentResponse = await page.request.get(currentHref!);
    expect(currentResponse.ok()).toBe(true);
    const current = await currentResponse.json();
    expect(current.run_id).not.toBe(record.run_id);
    expect(current.apply.state).toBe("committed");
    expect(current.apply.receipt_survived_restart).toBe(true);
    expect(current.apply.independent_tests_passed).toBe(26);
    expect(current.apply.repository_tests_passed).toBe(11);
    expect(current.primary_before_apply.inventory_matches).toBe(true);
    expect(current.primary_before_apply.cancel_left_unchanged).toBe(true);
    expect(current.native_ui.lower_diff_wheel_reachability).toBe("passed");
    await expect(page.getByText(`Recorded checkpoint MSI SHA256: ${current.msi_sha256}`, { exact: true })).toBeVisible();
    await expect(page.getByText(/This recording predates the newer Desktop polish build/)).toBeVisible();
    await page.getByRole("heading", { name: "September 8 recorded checkpoint" }).evaluate(element => element.scrollIntoView({ block: "start" }));
    await page.evaluate(() => window.scrollBy(0, -96));
    await page.screenshot({ path: testInfo.outputPath(`checkpoint-viewport-${width}.png`) });
    for (const name of ["Direct worktree record", "Earlier refusal record", "Launch failure record", "Review withheld record", "Earlier onboarding record", "Earlier CI candidate record"]) {
      const url = await page.getByRole("link", { name, exact: true }).getAttribute("href");
      expect(url).toBeTruthy();
      expect((await page.request.get(url!)).ok()).toBe(true);
    }
    await expect(page.getByText(/Host filesystem and network controls remained advisory/)).toBeVisible();
    const corpus = page.getByRole("region", { name: "The corpus", exact: true });
    await corpus.getByRole("heading", { name: "The corpus", exact: true }).evaluate(element => element.scrollIntoView({ block: "start" }));
    await page.evaluate(() => window.scrollBy(0, -96));
    if (width < 640) {
      // Each mobile value needs a readable visible label, not a hidden desktop header.
      for (const label of ["Files", "Source bytes", "Scaffold bytes", "Reduction"]) {
        const term = corpus.getByRole("term").filter({ hasText: new RegExp(`^${label}$`) }).first();
        await expect(term).toBeInViewport();
        const box = await term.boundingBox();
        expect(box?.width ?? 0).toBeGreaterThan(20);
        expect(box?.height ?? 0).toBeGreaterThan(12);
      }
    }
    await page.screenshot({ path: testInfo.outputPath(`corpus-viewport-${width}.png`) });
    await page.screenshot({ path: testInfo.outputPath(`checkpoint-full-${width}.png`), fullPage: true });
    await expectNoHorizontalOverflow(page);
  });
}

test("chroma aperture identity is monochrome with one static spectrum signature", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const heading = page.getByRole("heading", { level: 1 });
  const font = await page.locator("body").evaluate((el) => getComputedStyle(el).fontFamily);
  const headingColor = await heading.evaluate((el) => getComputedStyle(el).color);
  expect(font.toLowerCase()).toMatch(/satoshi|sora/);
  expect(font.toLowerCase()).not.toContain("geist");
  expect(headingColor).toBe("rgb(245, 245, 247)");
  await expect(page.locator(".nebula-bg, .chroma-glow")).toHaveCount(0);
  await expect(page.locator(".chroma-text")).toHaveCount(0);

  // The spectrum is a brand signature: exactly one instance, and it never animates.
  await expect(page.locator(".execution-trace")).toHaveCount(1);
  const animation = await page
    .locator(".execution-trace")
    .evaluate((el) => getComputedStyle(el).animationName);
  expect(animation).toBe("none");

  await expect(page.locator('header img[src*="logo"]')).toHaveCount(1);
  const download = page.locator("header").getByRole("link", { name: "Download", exact: true });
  await expect(download).toHaveCount(1);
  const cssText = await page.evaluate(() =>
    [...document.styleSheets]
      .flatMap((sheet) => {
        try {
          return [...sheet.cssRules].map((rule) => rule.cssText);
        } catch {
          return [];
        }
      })
      .join("\n"),
  );
  expect(cssText).toContain("--aperture-magenta");
});

test("the homepage no longer pins a scroll-jacked sequence", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  await expect(page.locator(".pin-spacer")).toHaveCount(0);
});

const PLAN_TIERS = [
  { title: "Pytxo Core", status: "Available now" },
  { title: "Cloud / Teams", status: "Not a default hosted product" },
] as const;

test("plans expose Core versus Cloud / Teams without subscribe tiers", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/plans");

  await expect(
    page.getByRole("heading", { level: 1, name: "Plans and availability" }),
  ).toBeVisible();

  const grid = page.getByTestId("plans-grid");
  const cells = grid.getByTestId("plan-cell");
  await expect(cells).toHaveCount(2);

  for (const tier of PLAN_TIERS) {
    await expect(page.locator('[data-slot="card-title"]', { hasText: tier.title })).toBeVisible();
    await expect(page.getByText(tier.status, { exact: true })).toBeVisible();
  }

  const layout = await cells.evaluateAll((nodes) => {
    const tops = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().top)))];
    const lefts = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().left)))];
    return { rowCount: tops.length, columnCount: lefts.length };
  });
  expect(layout.rowCount).toBe(1);
  expect(layout.columnCount).toBe(2);

  await expect(
    page.getByText(
      "Cloud execution and team entitlements are not a default hosted product. They work only in deployments that already have those services configured. Local Core does not require checkout.",
    ),
  ).toBeVisible();
  await expect(page.getByText("Subscribe to Pro")).toHaveCount(0);
  await expect(page.getByText("Subscribe to Max")).toHaveCount(0);
  await expect(page.getByText("Subscribe to Ultra")).toHaveCount(0);
  await expect(page.getByText("Pytxo Pro Cloud")).toHaveCount(0);
  await expect(page.getByText("Isolated sandbox service")).toHaveCount(0);
  await expect(page.getByText("Live billing")).toHaveCount(0);
  await expect(page.getByText("Hosted cloud sandboxes")).toHaveCount(0);
  await expect(page.getByText("Managed metered billing")).toHaveCount(0);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({ path: testInfo.outputPath("plans-1440.png"), fullPage: true });
});

test("plans remain complete and single-column on mobile", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/plans");

  const heading = page.getByRole("heading", { level: 1, name: "Plans and availability" });
  await expect(heading).toBeVisible();
  expect(await renderedLineCount(heading)).toBeLessThanOrEqual(2.2);

  const cells = page.getByTestId("plan-cell");
  await expect(cells).toHaveCount(2);
  for (const tier of PLAN_TIERS) {
    await expect(page.locator('[data-slot="card-title"]', { hasText: tier.title })).toBeVisible();
    await expect(page.getByText(tier.status, { exact: true })).toBeVisible();
  }

  const layout = await cells.evaluateAll((nodes) => {
    const tops = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().top)))];
    const lefts = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().left)))];
    return { rowCount: tops.length, columnCount: lefts.length };
  });
  expect(layout.rowCount).toBe(2);
  expect(layout.columnCount).toBe(1);

  await expect(page.getByText("Subscribe to Pro")).toHaveCount(0);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({ path: testInfo.outputPath("plans-390.png"), fullPage: true });
});
