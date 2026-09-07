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

async function contentLineCount(locator: ReturnType<Page["getByRole"]>) {
  return locator.evaluate((element) => {
    const range = document.createRange();
    range.selectNodeContents(element);
    const tops = [
      ...new Set([...range.getClientRects()].map((rect) => Math.round(rect.top))),
    ];
    return tops.length;
  });
}

test("desktop hero leads with the mission and current product evidence", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const heading = page.getByRole("heading", { level: 1 });
  const product = page.getByTestId("hero-product");
  const productImage = product.getByRole("img");

  await expect(heading).toHaveText("Agents do the work.You decide what lands.");
  await expect(page.getByTestId("marketing-hero").getByRole("link", { name: "Download Pytxo" })).toBeVisible();
  await expect(product).toBeInViewport();
  await expect(productImage).toHaveJSProperty("naturalWidth", 1600);
  expect(await renderedLineCount(heading)).toBeLessThanOrEqual(2.2);

  // The headline must stay inside its own grid column, never under the capture.
  const productBox = await product.boundingBox();
  const headingBox = await heading.boundingBox();
  expect(headingBox && productBox ? headingBox.x + headingBox.width : 0).toBeLessThanOrEqual(productBox?.x ?? 1440);
  expect(await heading.evaluate((element) => element.scrollWidth - element.clientWidth)).toBeLessThanOrEqual(1);
  await expectNoHorizontalOverflow(page);
});

test("hero states one primary action and an honest platform scope", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const hero = page.getByTestId("marketing-hero");
  const buttons = hero.getByRole("link").filter({ hasText: /Download Pytxo|Run your first mission/ });
  await expect(buttons).toHaveCount(2);
  await expect(hero.getByRole("link", { name: "Run your first mission" })).toHaveAttribute("href", "/docs/getting-started/first-mission");
  await expect(hero.getByText("Local Core needs no Pytxo account.", { exact: false })).toBeVisible();
  await expect(hero.getByText("Desktop ships for Windows today.")).toBeVisible();
});

test("mobile hero exposes the CTA and beginning of real product evidence", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");

  const hero = page.getByTestId("marketing-hero");
  const heading = hero.getByRole("heading", { level: 1 });
  const download = hero.getByRole("link", { name: "Download Pytxo" });

  await expect(heading).toBeVisible();
  await expect(download).toBeInViewport();
  expect(await renderedLineCount(heading)).toBeLessThanOrEqual(3.2);
  await expectNoHorizontalOverflow(page);
});

test("the homepage tells the seven-section narrative in order", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const order = await page.evaluate(() => {
    const ids = [
      "marketing-hero",
      "product-section",
      "boundary-section",
      "compatibility-section",
      "evidence-section",
      "get-it-section",
    ];
    return ids.map((id) => {
      const node = document.querySelector(`[data-testid="${id}"]`);
      return node ? Math.round(node.getBoundingClientRect().top + window.scrollY) : -1;
    });
  });

  expect(order.every((top) => top >= 0)).toBe(true);
  expect([...order].sort((a, b) => a - b)).toEqual(order);
  await expect(page.getByRole("heading", { name: "One agent first. More when it helps." })).toBeVisible();
});

test("the product section shows one capture large enough to read", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const section = page.getByTestId("product-section");
  const captures = section.getByRole("img");
  await expect(captures).toHaveCount(1);

  const box = await captures.first().boundingBox();
  expect(box?.width ?? 0).toBeGreaterThan(900);
  await expect(section.getByRole("heading", { name: "Commit boundary" })).toBeVisible();
  await expect(section.getByRole("heading", { name: "Enforcement receipt" })).toBeVisible();
});

test("the boundary section publishes every enforcement state, not just the good ones", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const section = page.getByTestId("boundary-section");
  await expect(section.getByText("Advisory only", { exact: true })).toBeVisible();
  await expect(section.getByText("Unavailable", { exact: true })).toBeVisible();
  await expect(section.getByText("Enforced", { exact: true }).first()).toBeVisible();
  await expect(section.getByText("Network", { exact: true })).toBeVisible();
});

test("compatibility names the real registry adapters and their commands", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const section = page.getByTestId("compatibility-section");
  for (const name of ["Claude Code", "OpenAI Codex", "Cursor Agent", "Antigravity"]) {
    await expect(section.getByText(name, { exact: true })).toBeVisible();
  }
  await expect(section.getByText("cursor-agent -p --trust", { exact: true })).toBeVisible();
  await expect(page.locator(".aperture-marquee")).toHaveCount(0);
  await expectNoHorizontalOverflow(page);
});

test("the published figure carries its non-claims and links to source data", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const section = page.getByTestId("evidence-section");
  await expect(section.getByTestId("evidence-figure")).toHaveText("82.9%");
  await expect(section.getByText("Not a model-token saving. Tokenizer output was not measured.")).toBeVisible();
  await expect(section.getByRole("link", { name: "Read the methodology" })).toBeVisible();
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
  test(`native evidence links resolve to the displayed artifact at ${width}px`, async ({ page }) => {
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
    for (const name of ["Direct worktree record", "Earlier refusal record", "Launch failure record", "Review withheld record"]) {
      const url = await page.getByRole("link", { name, exact: true }).getAttribute("href");
      expect(url).toBeTruthy();
      expect((await page.request.get(url!)).ok()).toBe(true);
    }
    await expect(page.getByText(/Host filesystem and network controls remained advisory/)).toBeVisible();
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
  { title: "Pytxo Pro Cloud", status: "Capability-gated" },
  { title: "Pytxo Max Swarm", status: "Configured deployments" },
  { title: "Pytxo Ultra", status: "Local ledger available" },
] as const;

test("plans expose capability gates before checkout", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/plans");

  await expect(
    page.getByRole("heading", { level: 1, name: "Plans and availability" }),
  ).toBeVisible();

  const grid = page.getByTestId("plans-grid");
  const cells = grid.getByTestId("plan-cell");
  await expect(cells).toHaveCount(4);

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
  expect(layout.columnCount).toBe(2);

  await expect(
    page.getByText(
      "Cloud execution is not a default hosted service yet. It works only in deployments with a configured cloud dispatcher.",
    ),
  ).toBeVisible();
  await expect(
    page.getByText(
      "Ultra mode includes local metering hooks. Managed inference and Link reconciliation are not end-to-end on the default path.",
    ),
  ).toBeVisible();
  await expect(page.getByText("Sandbox service when configured")).toBeVisible();
  await expect(page.getByText("Isolated sandbox service")).toHaveCount(0);
  await expect(page.getByText("Live billing")).toHaveCount(0);
  await expect(page.getByText("Hosted cloud sandboxes")).toHaveCount(0);
  await expect(page.getByText("Managed metered billing")).toHaveCount(0);
  const checkoutConfigured =
    (await page.getByRole("link", { name: "Subscribe to Pro" }).count()) > 0;
  if (checkoutConfigured) {
    await expect(page.getByRole("link", { name: "Subscribe to Pro" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Subscribe to Max" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Subscribe to Ultra" })).toBeVisible();
  } else {
    await expect(
      page.getByRole("button", { name: "Account checkout not configured" }),
    ).toHaveCount(3);
  }
  await expectNoHorizontalOverflow(page);
});

test("plans remain complete and single-column on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/plans");

  const heading = page.getByRole("heading", { level: 1, name: "Plans and availability" });
  await expect(heading).toBeVisible();
  expect(await renderedLineCount(heading)).toBeLessThanOrEqual(2.2);

  const cells = page.getByTestId("plan-cell");
  await expect(cells).toHaveCount(4);
  for (const tier of PLAN_TIERS) {
    await expect(page.locator('[data-slot="card-title"]', { hasText: tier.title })).toBeVisible();
    await expect(page.getByText(tier.status, { exact: true })).toBeVisible();
  }

  const layout = await cells.evaluateAll((nodes) => {
    const tops = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().top)))];
    const lefts = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().left)))];
    return { rowCount: tops.length, columnCount: lefts.length };
  });
  expect(layout.rowCount).toBe(4);
  expect(layout.columnCount).toBe(1);

  const checkoutConfigured =
    (await page.getByRole("link", { name: "Subscribe to Pro" }).count()) > 0;
  if (checkoutConfigured) {
    for (const name of ["Subscribe to Pro", "Subscribe to Max", "Subscribe to Ultra"]) {
      const link = page.getByRole("link", { name });
      await expect(link).toBeVisible();
      expect(await contentLineCount(link)).toBe(1);
    }
  } else {
    await expect(
      page.getByRole("button", { name: "Account checkout not configured" }),
    ).toHaveCount(3);
  }
  await expectNoHorizontalOverflow(page);
});
