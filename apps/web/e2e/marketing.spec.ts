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
    await expect(hero.getByRole("heading", { level: 1 })).toContainText("Many agents.");
    await expect(hero.getByRole("heading", { level: 1 })).toContainText("One verified change.");
    await expect(hero.getByText(/Pytxo splits it across the coding agents you choose/)).toBeVisible();
    await expect(hero.getByRole("link", { name: "Download for Windows" })).toBeInViewport();
    await expect(hero.getByText(/Mixed-agent runs arrive in/)).toHaveCount(0);
    await expect(hero.getByRole("link", { name: /See how it works/ })).toHaveAttribute("href", "#how");
    const agents = hero.getByRole("list", { name: "Agent CLIs Pytxo runs" });
    for (const name of ["Codex", "Claude Code", "Cursor", "OpenCode", "Antigravity"]) await expect(agents).toContainText(name);
    await expect(agents.locator('img[src*="antigravity"]')).toHaveCount(1);
    await expect(hero.getByTestId("hero-fleet").getByRole("img")).toHaveJSProperty("naturalWidth", 1600);
    const how = page.getByTestId("how-it-works");
    await expect(how.getByRole("listitem")).toHaveCount(3);
    // Step images load lazily; each must load once it is scrolled into view.
    for (const image of await how.getByRole("img").all()) { await image.scrollIntoViewIfNeeded(); await expect(image).toHaveJSProperty("naturalWidth", 1600); }
    await expect(how).toContainText("Screens are captures of the v1.2.2 Desktop");
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`launch-home-${width}.png`), fullPage: true });
    await hero.getByRole("link", { name: "Download for Windows" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "Download" })).toBeVisible();
    await expect(page.getByText(/download below installs the current public/)).toHaveCount(0);
    await expect(page.getByText("v1.2.2").first()).toBeVisible();
    await expect(page.getByRole("link", { name: "Pytxo-dev/pytxo" })).toHaveAttribute("href", "https://github.com/Pytxo-dev/pytxo");
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

test("homepage leads with the outcome, then how, why, proof and requirements", async ({ page }) => {
  await page.goto("/");
  const ids = ["marketing-hero", "how-it-works", "compare-section", "proof-section", "compatibility-section", "faq-section", "get-it-section"];
  const tops = await Promise.all(ids.map(id => page.getByTestId(id).evaluate(el => el.getBoundingClientRect().top)));
  expect([...tops].sort((a, b) => a - b)).toEqual(tops);
  await expect(page.getByTestId("compare-section").getByRole("row")).toHaveCount(6);
  await expect(page.getByTestId("compare-section")).toContainText("Exactly the reviewed files, refused if the project moved");
  const proof = page.getByTestId("proof-section");
  await expect(proof).toContainText("6/6");
  await expect(proof).toContainText("OpenCode and Antigravity finished their tasks without changing files");
  await expect(proof.getByRole("link", { name: /Read the run record/ })).toHaveAttribute("href", "/evidence#fleet");
  await expect(page.getByTestId("compatibility-section")).toContainText("your shell's API keys are not passed to them");
  await expect(page.getByTestId("faq-section")).toContainText("Does Pytxo sandbox everything an agent does?");
  await expect(page.locator(".aperture-marquee")).toHaveCount(0);
});

test("homepage FAQ answers open and close by keyboard", async ({ page }) => {
  await page.goto("/");
  const question = page.getByRole("button", { name: "What if my project changes after I review?" });
  await question.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByText("Apply refuses and nothing is written.", { exact: false })).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.getByText("Apply refuses and nothing is written.", { exact: false })).toBeHidden();
});

test("the evidence page states the corpus, caveats, and source access limit", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/evidence");

  await expect(page.getByRole("heading", { level: 1, name: "Evidence and its limits." })).toBeVisible();
  await expect(page.getByRole("heading", { name: "What it does not claim" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Caveats that weaken the number" })).toBeVisible();
  await expect(page.getByText(/worktree was dirty at capture time/)).toBeVisible();
  await expect(page.getByText(/committed in the public source repository/)).toBeVisible();
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

test("chroma identity keeps the spectrum static and limited to its two signatures", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const heading = page.getByRole("heading", { level: 1 });
  const font = await page.locator("body").evaluate((el) => getComputedStyle(el).fontFamily);
  const headingColor = await heading.evaluate((el) => getComputedStyle(el).color);
  expect(font.toLowerCase()).toMatch(/satoshi|sora/);
  expect(font.toLowerCase()).not.toContain("geist");
  expect(headingColor).toBe("rgb(245, 245, 247)");
  await expect(page.locator(".nebula-bg, .chroma-glow")).toHaveCount(0);
  // The approved fleet hero carries the spectrum once in its headline; it never animates.
  await expect(page.locator(".chroma-text")).toHaveCount(1);
  await expect(page.locator(".chroma-text")).toHaveText("One verified change.");
  expect(await page.locator(".chroma-text").evaluate((el) => getComputedStyle(el).animationName)).toBe("none");


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
