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
    await expect(hero.getByRole("heading", { level: 1 })).toHaveText("Clear your agent PR pile in one command.");
    await expect(hero.getByText(/lands the ones that pass/)).toBeVisible();
    await expect(hero.getByRole("button", { name: "Copy npx pytxo demo" })).toBeInViewport({ ratio: 1 });
    await expect(hero.getByRole("link", { name: "Source on GitHub" })).toHaveAttribute("href", "https://github.com/Pytxo-dev/pytxo");
    // The demo shows the recorded bytes of a real run, not a retyped or illustrated one.
    const recordUrl = "/evidence/demo-landed-2.0.2-2026-11-12.json";
    const record = await (await page.request.get(recordUrl)).json();
    const recorded = record.output.trimEnd().split("\n");
    expect(record.command).toBe("npx pytxo demo");
    expect(record.output).toContain("LANDED   2 of 4 branches are on main now.");
    expect(record.output).toContain("left out codex/add-discount   uses price, which claude/rename-price renamed to unitPrice (src/discount.js:1)");
    expect(record.line_ms).toHaveLength(recorded.length);
    const demo = page.getByTestId("recorded-demo");
    // Every line of the record is on the page, in order. The fix prompt's body waits inside its disclosure.
    const lines = await demo.getByTestId("recorded-output").locator(":scope > div, :scope > details > *").allTextContents();
    expect(lines).toEqual([`$ ${record.command}`, ...recorded]);
    const fix = demo.locator("details");
    await expect(fix.getByText("Failing output:")).toBeHidden();
    await fix.locator("summary").click();
    await expect(fix.getByText("Failing output:")).toBeVisible();
    await expect(demo.getByRole("link", { name: /^Recorded output/ })).toHaveAttribute("href", recordUrl);
    // The ledger is what the same record says happened to each branch, the landed ones first.
    const decide = page.getByTestId("decide-section");
    const rows = decide.getByRole("row");
    await expect(rows).toHaveCount(4);
    for (const [i, [branch, outcome]] of [
      ["copilot/add-tax", "landed"],
      ["cursor/add-guide", "landed"],
      ["claude/rename-price", "left out"],
      ["codex/add-discount", "left out"],
    ].entries()) {
      expect(recorded.some((line: string) => line.startsWith(outcome) && line.slice(outcome.length).trimStart().split(" ")[0] === branch)).toBe(true);
      await expect(rows.nth(i).getByRole("rowheader")).toHaveText(branch);
      await expect(rows.nth(i).getByRole("cell")).toHaveText(outcome);
    }
    await expect(decide.getByText("Two landed. Two left out, with the reason.")).toBeVisible();
    await expect(decide.getByRole("link", { name: /^Recorded output/ })).toHaveAttribute("href", recordUrl);
    const where = page.getByTestId("where-section");
    await expect(where.locator("code")).toHaveText(["npx pytxo --apply", "npx pytxo hook", "uses: Pytxo-dev/pytxo/actions/land@v2"]);
    for (const [name, href] of [
      ["Claude Code plugin or the Codex hook", "/docs/getting-started/agent-hooks"],
      ["GitHub Action", "/docs/getting-started/land-in-ci"],
    ]) {
      await expect(where.getByRole("link", { name })).toHaveAttribute("href", href);
      expect((await page.request.get(href)).status()).toBe(200);
    }
    await expect(page.getByTestId("close-section").getByRole("button", { name: "Copy npx pytxo demo" })).toBeVisible();
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`launch-home-${width}.png`), fullPage: true });
    // The frozen Desktop download stays one click away, in the footer.
    await page.locator("footer").getByRole("link", { name: "Desktop (frozen)" }).click();
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
    // Next steps lead to the check's guides, not to the frozen 1.x flows.
    const next = page.getByTestId("next-steps");
    await expect(next.getByRole("link", { name: "Quickstart" })).toHaveAttribute("href", "/docs/getting-started/quickstart");
    await expect(next.getByRole("link", { name: "First mission" })).toHaveCount(0);
    await expectNoHorizontalOverflow(page);
    await page.screenshot({ path: testInfo.outputPath(`launch-download-${width}.png`), fullPage: true });
  });
}

test("plans present current availability without a public subscription checkout", async ({ page }) => {
  await page.goto("/plans");
  await expect(page.getByRole("heading", { level: 1, name: "Plans" })).toBeVisible();
  await expect(page.locator("main").getByText("Free on your machine and on public repositories.", { exact: false })).toBeVisible();
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

test("homepage asks the question, then shows the recorded run, how it decides and where it runs", async ({ page }) => {
  await page.goto("/");
  const ids = ["marketing-hero", "recorded-demo", "decide-section", "where-section", "close-section"];
  const tops = await Promise.all(ids.map(id => page.getByTestId(id).evaluate(el => el.getBoundingClientRect().top)));
  expect([...tops].sort((a, b) => a - b)).toEqual(tops);
  await expect(page.locator("main")).not.toContainText(/without the token|has no token/i);
  // Newcomers see none of the frozen 1.x vocabulary.
  await expect(page.locator("main")).not.toContainText(/\b(hypervisor|control plane|telemetry|orbit|galaxy|deepspace|supernova|signal core|blast shield|missions?|fleets?)\b/i);
  await expect(page.locator(".aperture-marquee")).toHaveCount(0);
});

test("the questions live in the docs, and the Action's answer says what a fork's run gets", async ({ page }) => {
  await page.goto("/docs/reference/faq");
  await expect(page.getByRole("heading", { level: 1, name: "Questions" })).toBeVisible();
  const article = page.locator("article");
  for (const text of [
    "Does it send my code anywhere?",
    "The merge and the tests happen in a throwaway worktree.",
    "Is the GitHub Action safe on public repositories?",
    "checked in its own run, with GitHub's read-only token and no secrets",
  ]) {
    await expect(article).toContainText(text);
  }
  await expect(article).not.toContainText(/without the token|has no token/i);
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
    const record = await (await page.request.get("/evidence/pybench-2.0.1-2026-11-05.json")).json();
    expect(record.schema).toBe("pybench/v1");
    const reference = page.getByTestId("bench-reference");
    await expect(reference).toContainText(`${record.land.summary.pytxo_prevented}/${record.land.summary.naive_landed_broken}`);
    await expect(reference.getByRole("row")).toHaveCount(record.land.scenarios.length + 1 + (record.overhead ? record.overhead.rows.length + 1 : 0));
    await expect(reference).toContainText("lands a broken main");
    // The run says which build it measured and when.
    await expect(reference).toContainText(`pytxo ${record.pytxo_version}`);
    await expect(reference).toContainText(record.created_at.slice(0, 10));
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

test("the site keeps to ink, paper and one gradient", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/");

  const font = await page.locator("body").evaluate((el) => getComputedStyle(el).fontFamily);
  expect(font.toLowerCase()).toContain("sora");
  expect(font.toLowerCase()).not.toContain("geist");
  await expect(page.getByRole("heading", { level: 1 })).toHaveCSS("color", "rgb(236, 234, 228)");
  // The page is the ink the hero's canvas clears to, so the hero shows no band.
  await expect(page.locator('[data-site="v2"]')).toHaveCSS("background-color", "rgb(8, 8, 10)");
  await expect(page.locator(".nebula-bg, .chroma-glow, .chroma-text")).toHaveCount(0);

  const header = page.locator("header");
  await expect(header.getByRole("link", { name: "pytxo home" }).locator("svg")).toHaveCount(1);
  await expect(header.getByRole("link", { name: "Docs", exact: true })).toHaveAttribute("href", "/docs");
  await expect(header.getByRole("button", { name: "Copy npx pytxo demo" })).toBeVisible();
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
    await expect(page.getByRole("heading", { level: 2, name: tier.title, exact: true })).toBeVisible();
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
    await expect(page.getByRole("heading", { level: 2, name: tier.title, exact: true })).toBeVisible();
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
