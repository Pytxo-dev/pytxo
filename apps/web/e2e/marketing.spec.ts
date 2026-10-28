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
    await expect(hero.getByRole("heading", { level: 1 })).toHaveText("Each agent's branch passes. Do they pass together?");
    // The hero shows the recorded bytes of a real run, not a retyped or illustrated one.
    const record = await (await page.request.get("/evidence/demo-blocked-2.0.0-2026-10-28.json")).json();
    expect(record.command).toBe("npx pytxo demo");
    expect(record.output).toContain("BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.");
    const capture = hero.getByTestId("hero-capture");
    expect(await capture.locator("pre").textContent()).toBe(`$ ${record.command}\n${record.output.trimEnd()}\n`);
    await expect(capture.getByText("BLOCKED  claude/rename-price and codex/add-discount pass npm test alone but fail together.")).toBeInViewport();
    await expect(capture.getByRole("link", { name: "Raw output" })).toHaveAttribute("href", "/evidence/demo-blocked-2.0.0-2026-10-28.json");
    await expect(hero.getByText(/runs your tests on the combined result, and blocks the merge if it breaks/)).toBeVisible();
    await expect(hero.locator("pre").filter({ hasText: /^npx pytxo demo$/ })).toBeVisible();
    await expect(hero.getByRole("button", { name: "Copy the demo command" })).toBeVisible();
    await expect(hero.getByRole("link", { name: "Source on GitHub" })).toHaveAttribute("href", "https://github.com/Pytxo-dev/pytxo");
    await expect(hero.getByRole("link", { name: "Add it to CI" })).toHaveAttribute("href", "/#ci");
    await expect(hero.getByRole("link", { name: "mattbaconz" })).toHaveAttribute("href", "https://github.com/mattbaconz");
    await expect(hero.getByRole("link", { name: "lukeed/mri" })).toHaveAttribute("href", "/docs/getting-started/first-landing#3-if-the-decision-is-blocked");
    expect(await (await page.request.get("/docs/getting-started/first-landing")).text()).toContain('id="3-if-the-decision-is-blocked"');
    const how = page.getByTestId("how-it-works");
    await expect(how.getByRole("listitem")).toHaveCount(3);
    // The step snippets are lines of the same recorded run.
    await expect(how).toContainText("Checking claude/rename-price and codex/add-discount against main with npm test (from package.json)...");
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`launch-home-${width}.png`), fullPage: true });
    await hero.getByRole("link", { name: "Add it to CI" }).click();
    const ci = page.getByTestId("ci-section");
    await expect(ci.getByRole("heading", { level: 2, name: "One line in CI." })).toBeInViewport();
    await expect(ci.locator("pre")).toContainText("uses: Pytxo-dev/pytxo/actions/land@v2");
    await expect(ci.getByRole("link", { name: /Set it up/ })).toHaveAttribute("href", "/docs/getting-started/land-in-ci");
    // The frozen Desktop download stays one click away, in the footer.
    await page.locator("footer").getByRole("link", { name: "Pytxo Desktop (frozen)" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "Download" })).toBeVisible();
    await expect(page.getByText(/download below installs the current public/)).toHaveCount(0);
    await expect(page.getByText(/^v\d+\.\d+\.\d+$/).first()).toBeVisible();
    await expect(page.getByRole("link", { name: "Pytxo-dev/pytxo" })).toHaveAttribute("href", "https://github.com/Pytxo-dev/pytxo");
    await expect(page.getByRole("button", { name: "Not yet" })).toHaveCount(2);
    await expect(page.getByText("The CLI does not Apply repository changes")).toHaveCount(0);
    // The CLI section sells the check, not the frozen 1.x commands.
    await expect(page.getByText("Checks your agents' branches together.", { exact: false })).toBeVisible();
    await expect(page.getByText(/Runs, Review and Apply/)).toHaveCount(0);
    await expect(page.getByText("planning, waves, isolation, and Apply")).toHaveCount(0);
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`launch-download-${width}.png`), fullPage: true });
  });
}

test("plans present current availability without a public subscription checkout", async ({ page }) => {
  await page.goto("/plans");
  await expect(page.getByRole("heading", { level: 1, name: "Plans" })).toBeVisible();
  await expect(page.getByText("Free on your machine and on public repositories.", { exact: false })).toBeVisible();
  await expect(page.getByRole("link", { name: /Subscribe to|Get Pro/i })).toHaveCount(0);
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
  const ids = ["marketing-hero", "how-it-works", "ci-section", "compare-section", "proof-section", "compatibility-section", "faq-section", "get-it-section"];
  const tops = await Promise.all(ids.map(id => page.getByTestId(id).evaluate(el => el.getBoundingClientRect().top)));
  expect([...tops].sort((a, b) => a - b)).toEqual(tops);
  await expect(page.getByTestId("compare-section").getByRole("row")).toHaveCount(5);
  await expect(page.getByTestId("compare-section")).toContainText("Before you merge anything");
  const proof = page.getByTestId("proof-section");
  await expect(proof.getByTestId("proof-bench")).toContainText("4/4");
  await expect(proof.getByTestId("proof-bench")).toContainText("12/12");
  await expect(proof).toContainText("Checks passing is evidence, not proof.");
  await expect(proof.getByRole("link", { name: /Pybench results/ })).toHaveAttribute("href", "/bench");
  await expect(proof.getByRole("link", { name: /All evidence and limits/ })).toHaveAttribute("href", "/docs/evidence");
  await expect(page.getByTestId("marketing-hero")).toContainText("Each passes npm test on its own. Together the test fails, so the check blocks.");
  await expect(page.getByTestId("compatibility-section")).toContainText("Pytxo calls no model and sends nothing anywhere.");
  await expect(page.getByTestId("faq-section")).toContainText("Does it send my code anywhere?");
  await expect(page.getByTestId("get-it-section").locator("pre").filter({ hasText: /^npx pytxo demo$/ })).toBeVisible();
  // The Action's safety copy says what a fork's run gets, not that the tests never see the token.
  await expect(page.getByTestId("ci-section")).toContainText("checked in its own run, with GitHub's read-only token and no secrets");
  await expect(page.locator("main")).not.toContainText(/without the token|has no token/i);
  // A pull request that is not to blame for a break does not fail.
  await expect(page.getByTestId("ci-section")).toContainText("when only others are to blame, the comment names them and the job passes");
  // Newcomers see none of the frozen 1.x vocabulary.
  await expect(page.locator("main")).not.toContainText(/\b(hypervisor|control plane|telemetry|orbit|galaxy|deepspace|supernova|signal core|blast shield|missions?|fleets?)\b/i);
  await expect(page.locator(".aperture-marquee")).toHaveCount(0);
});

for (const width of [1440, 390]) {
  test(`every FAQ answer opens fully at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto("/");
    const triggers = page.getByTestId("faq-section").getByRole("button");
    const count = await triggers.count();
    expect(count).toBeGreaterThan(5);
    for (let i = 0; i < count; i++) {
      const trigger = triggers.nth(i);
      await trigger.click();
      const region = page.getByTestId("faq-section").getByRole("region").first();
      await expect(region).toBeVisible();
      await region.evaluate((el) => Promise.all(el.getAnimations({ subtree: true }).map((a) => a.finished)));
      const clipped = await region.evaluate((el) => {
        const inner = el.firstElementChild as HTMLElement | null;
        return el.scrollHeight - el.clientHeight > 1 || (inner ? inner.scrollHeight - inner.clientHeight > 1 : false);
      });
      expect(clipped, (await trigger.textContent()) ?? "faq answer").toBe(false);
      await trigger.click();
    }
  });
}

test("homepage FAQ answers open and close by keyboard", async ({ page }) => {
  await page.goto("/");
  const question = page.getByRole("button", { name: "Does it change my repository?" });
  await question.focus();
  await page.keyboard.press("Enter");
  await expect(page.getByText("The merge and the tests happen in a throwaway worktree.", { exact: false })).toBeVisible();
  await page.keyboard.press("Enter");
  await expect(page.getByText("The merge and the tests happen in a throwaway worktree.", { exact: false })).toBeHidden();
});

test("the Action's FAQ answer says what a fork's run gets, not that the tests never see the token", async ({ page }) => {
  // A closed answer is not in the DOM, so the guards on main cannot see it.
  await page.goto("/");
  const question = "Is the GitHub Action safe on public repositories?";
  await page.getByRole("button", { name: question }).click();
  const answer = page.getByTestId("faq-section").getByRole("region", { name: question });
  await expect(answer).toContainText("checked in its own run, with GitHub's read-only token and no secrets");
  await expect(answer).not.toContainText(/without the token|has no token/i);
});

test("evidence moved into the docs and keeps every record downloadable", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/evidence");
  await expect(page).toHaveURL(/\/docs\/evidence$/);
  await expect(page.getByRole("heading", { level: 1, name: "Evidence" })).toBeVisible();

  await page.goto("/docs/evidence/live-runs");
  await expect(page.getByText("Checks passing is not correctness.", { exact: false })).toBeVisible();
  for (const href of [
    "/evidence/fleet-live-2026-10-10.json",
    "/evidence/fleet-native-2026-10-02.json",
    "/evidence/astra-native-codex-2026-09-07.json",
    "/evidence/astra-refused-run-2026-09-07.json",
    "/evidence/astra-transport-failure-2026-09-07.json",
    "/evidence/astra-review-withheld-2026-09-07.json",
  ]) {
    await expect(page.locator(`a[href="${href}"]`).first()).toBeVisible();
    const response = await page.request.get(href);
    expect(response.ok(), href).toBe(true);
    expect(response.headers()["content-type"]).toContain("application/json");
  }
  const live = await (await page.request.get("/evidence/fleet-live-2026-10-10.json")).json();
  expect(live.apply.files_written).toBe(live.review.files);
  expect(live.fixture_tests_after_apply.failed).toBe(0);
  await expect(page.getByText(`${live.apply.files_written}/${live.review.files}`, { exact: true })).toBeVisible();

  await page.goto("/docs/evidence/first-use");
  const firstUse = await (await page.request.get("/evidence/first-use-2026-10-15.json")).json();
  const decisions = firstUse.runs[1].steps.filter((step: { decision?: string }) => step.decision).map((step: { decision: string }) => step.decision);
  expect(decisions).toEqual(["BLOCKED", "READY", "LANDED"]);
  expect(JSON.stringify(firstUse.receipts)).not.toMatch(/Users\\\\(?!<user>)/);
  await expect(page.locator('a[href="/evidence/first-use-2026-10-15.json"]')).toBeVisible();
  await expect(page.getByText("96.1 s", { exact: false }).first()).toBeVisible();
  const take = await (await page.request.get("/evidence/first-use-film-1.3.1-2026-10-16.json")).json();
  expect(take.steps.filter((step: { decision?: string }) => step.decision).map((step: { decision: string }) => step.decision)).toEqual(["BLOCKED", "READY", "LANDED"]);
  expect(Object.values(take.receipts).every((receipt) => (receipt as { predicate: { pytxo_version: string } }).predicate.pytxo_version === "1.3.1")).toBe(true);
  await expect(page.locator('a[href="/evidence/first-use-film-1.3.1-2026-10-16.json"]')).toBeVisible();

  await page.goto("/docs/evidence/signal-core");
  await expect(page.getByText("Not a model-token saving.", { exact: false })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(1);
});

for (const width of [1440, 390]) {
  test(`pybench shows the reference run from its record at ${width}px`, async ({ page }) => {
    await page.setViewportSize({ width, height: 900 });
    await page.goto("/bench");
    const record = await (await page.request.get("/evidence/pybench-1.3.0-2026-10-10.json")).json();
    expect(record.schema).toBe("pybench/v1");
    const reference = page.getByTestId("bench-reference");
    await expect(reference).toContainText(`${record.land.summary.pytxo_prevented}/${record.land.summary.naive_landed_broken}`);
    await expect(reference.getByRole("row")).toHaveCount(record.land.scenarios.length + 1 + (record.overhead ? record.overhead.rows.length + 1 : 0));
    await expect(reference).toContainText("lands a broken main");
    await expect(page.getByTestId("bench-board")).toBeVisible();
    await expectNoHorizontalOverflow(page);
  });
}

test("account connect refuses incomplete links and receipt ids are validated", async ({ page }) => {
  await page.goto("/account/connect?client=cli&state=short");
  await expect(page.locator("main")).toContainText(/incomplete or expired|not available on this site/);
  const missing = await page.goto("/r/not-a-receipt");
  expect(missing?.status()).toBe(404);
});

test("chroma identity keeps the spectrum to the logo", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const heading = page.getByRole("heading", { level: 1 });
  const font = await page.locator("body").evaluate((el) => getComputedStyle(el).fontFamily);
  const headingColor = await heading.evaluate((el) => getComputedStyle(el).color);
  expect(font.toLowerCase()).toMatch(/satoshi|sora/);
  expect(font.toLowerCase()).not.toContain("geist");
  expect(headingColor).toBe("rgb(245, 245, 247)");
  await expect(page.locator(".nebula-bg, .chroma-glow")).toHaveCount(0);
  // The headline is plain text; the spectrum stays in the logo.
  await expect(page.locator(".chroma-text")).toHaveCount(0);


  await expect(page.locator('header img[src*="logo"]')).toHaveCount(1);
  const getStarted = page.locator("header").getByRole("link", { name: "Get started", exact: true });
  await expect(getStarted).toHaveCount(1);
  await expect(getStarted).toHaveAttribute("href", "/docs/getting-started/quickstart");
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
  { title: "Free", status: "Free · available now" },
  { title: "Pro", status: "$29 once · not on sale yet" },
  { title: "Team", status: "Planned" },
] as const;

test("plans expose free use, Pro for private repositories and planned Team", async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/plans");

  await expect(
    page.getByRole("heading", { level: 1, name: "Plans", exact: true }),
  ).toBeVisible();

  const grid = page.getByTestId("plans-grid");
  const cells = grid.getByTestId("plan-cell");
  await expect(cells).toHaveCount(3);

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
  expect(layout.columnCount).toBe(3);

  await expect(page.getByText("Everything that runs on your machine is free and stays free.", { exact: false })).toBeVisible();
  await expect(page.getByText("Not available to buy.", { exact: false })).toBeVisible();
  await expect(page.getByText("If development stops, Pro unlocks for everyone.", { exact: false })).toBeVisible();
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

  const heading = page.getByRole("heading", { level: 1, name: "Plans", exact: true });
  await expect(heading).toBeVisible();
  expect(await renderedLineCount(heading)).toBeLessThanOrEqual(2.2);

  const cells = page.getByTestId("plan-cell");
  await expect(cells).toHaveCount(3);
  for (const tier of PLAN_TIERS) {
    await expect(page.locator('[data-slot="card-title"]', { hasText: tier.title })).toBeVisible();
    await expect(page.getByText(tier.status, { exact: true })).toBeVisible();
  }

  const layout = await cells.evaluateAll((nodes) => {
    const tops = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().top)))];
    const lefts = [...new Set(nodes.map((node) => Math.round(node.getBoundingClientRect().left)))];
    return { rowCount: tops.length, columnCount: lefts.length };
  });
  expect(layout.rowCount).toBe(3);
  expect(layout.columnCount).toBe(1);

  await expect(page.getByText("Subscribe to Pro")).toHaveCount(0);
  await expectNoHorizontalOverflow(page);
  await page.screenshot({ path: testInfo.outputPath("plans-390.png"), fullPage: true });
});

test("terms, privacy and refunds are linked from every page footer", async ({ page }) => {
  for (const [path, heading, phrase] of [
    ["/terms", "Terms", "Dodo Payments, our merchant of record"],
    ["/privacy", "Privacy", "send no telemetry"],
    ["/refunds", "Refunds", "fully refundable for 14 days"],
  ] as const) {
    await page.goto(path);
    await expect(page.getByRole("heading", { level: 1, name: heading, exact: true })).toBeVisible();
    await expect(page.getByText(phrase, { exact: false })).toBeVisible();
    await expectNoHorizontalOverflow(page);
  }
  await page.goto("/");
  const legal = page.getByRole("navigation", { name: "Legal" });
  for (const name of ["Terms", "Privacy", "Refunds"]) {
    await expect(legal.getByRole("link", { name, exact: true })).toBeVisible();
  }
});
