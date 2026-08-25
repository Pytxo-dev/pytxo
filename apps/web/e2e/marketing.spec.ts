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

  const heading = page.getByRole("heading", {
    level: 1,
    name: "Coordinate coding agents. Review one result.",
  });
  const product = page.getByTestId("hero-product");
  const productImage = product.getByRole("img");

  await expect(heading).toBeVisible();
  await expect(
    page.getByTestId("marketing-hero").getByRole("link", { name: "Download Desktop" }),
  ).toBeVisible();
  await expect(product).toBeInViewport();
  await expect(productImage).toHaveJSProperty("naturalWidth", 1600);
  expect(await renderedLineCount(heading)).toBeLessThanOrEqual(2.2);

  const productBox = await product.boundingBox();
  expect(productBox?.y).toBeLessThan(260);
  expect(productBox ? productBox.y + productBox.height : 901).toBeLessThan(650);
  const proof = page.getByRole("region", {
    name: "Supported tools and measured proof",
  });
  await expect(proof.getByText("82.9%", { exact: true })).toBeVisible();
  await expect(
    proof.getByRole("link", {
      name: "82.9% measured scaffold-byte reduction across 185 tracked production files. This is not a model-token or task-success claim.",
    }),
  ).toBeVisible();
  await expectNoHorizontalOverflow(page);
});

test("mobile hero exposes the CTA and beginning of real product evidence", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");

  const hero = page.getByTestId("marketing-hero");
  const heading = hero.getByRole("heading", { level: 1 });
  const download = hero.getByRole("link", { name: "Download" });
  const product = page.getByTestId("hero-product");

  await expect(heading).toBeVisible();
  await expect(download).toBeInViewport();
  await expect(product).toBeInViewport();
  expect(await renderedLineCount(heading)).toBeLessThanOrEqual(3.2);

  const productBox = await product.boundingBox();
  expect(productBox?.y).toBeLessThan(760);
  expect(productBox?.width).toBeLessThanOrEqual(358);
  await expectNoHorizontalOverflow(page);
});

test("the product story is complete when reduced motion is requested", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");

  const story = page.getByTestId("product-story");
  await expect(story.getByRole("heading", { name: "Set ownership before dispatch" })).toBeVisible();
  await expect(story.getByRole("heading", { name: "Review the prepared package" })).toBeVisible();
  await expect(story.getByRole("heading", { name: "Apply exactly what you reviewed" })).toBeVisible();
  await expect(page.locator(".pin-spacer")).toHaveCount(0);

  const styles = await story.locator("[data-story-card]").evaluateAll((cards) =>
    cards.map((card) => {
      const style = getComputedStyle(card);
      return { opacity: style.opacity, transform: style.transform };
    }),
  );
  expect(styles).toEqual([
    { opacity: "1", transform: "none" },
    { opacity: "1", transform: "none" },
    { opacity: "1", transform: "none" },
  ]);
});

test("agent readiness uses the current Agents product capture", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const section = page.getByRole("region", { name: "Keep the accounts you already trust" });
  await expect(section).toBeVisible();
  await expect(section.getByText("Agent sessions stay vendor-owned")).toBeVisible();
  await expect(section.getByText("API billing stays separate")).toBeVisible();
  await expect(section.getByText("Readiness checks are non-billable")).toBeVisible();
  const image = page.getByTestId("agent-readiness-product").getByRole("img");
  await image.scrollIntoViewIfNeeded();
  await expect(image).toHaveJSProperty("naturalWidth", 1600);
  await expectNoHorizontalOverflow(page);
});

test("chassis identity is plex metal, not geist nebula", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const heading = page.getByRole("heading", {
    level: 1,
    name: "Coordinate coding agents. Review one result.",
  });
  const font = await page.locator("body").evaluate((el) => getComputedStyle(el).fontFamily);
  const headingColor = await heading.evaluate((el) => getComputedStyle(el).color);
  expect(font.toLowerCase()).toContain("ibm plex");
  expect(font.toLowerCase()).not.toContain("geist");
  expect(headingColor).toBe("rgb(230, 232, 236)");
  await expect(page.locator(".nebula-bg, .chroma-text, .chroma-glow")).toHaveCount(0);
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
  expect(cssText).not.toContain("--brand-violet");
});

test("desktop storytelling keeps one focused pinned sequence", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  await expect(page.locator(".pin-spacer")).toHaveCount(1);
  await expect(page.getByTestId("product-story")).toBeVisible();
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
