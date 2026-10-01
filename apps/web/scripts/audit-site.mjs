import { chromium } from "@playwright/test";

const BASE = process.env.SHOOT_BASE ?? "http://127.0.0.1:3119";

function luminance([r, g, b]) {
  const channel = (value) => {
    const v = value / 255;
    return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(r) + 0.7152 * channel(g) + 0.0722 * channel(b);
}

function contrast(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

function parseRgb(value) {
  const parts = value.match(/[\d.]+/g);
  return parts ? parts.slice(0, 3).map(Number) : null;
}

const browser = await chromium.launch({ channel: process.env.PLAYWRIGHT_CHANNEL || "chromium" });
const session = { storageState: process.env.PLAYWRIGHT_STORAGE_STATE || undefined };
const failures = [];

// Acceptance criterion: the hero headline's glyph box must not intersect the
// product frame at any of the three reference widths.
for (const width of [1280, 1440, 1920]) {
  const context = await browser.newContext({ ...session, viewport: { width, height: 900 } });
  const page = await context.newPage();
  await page.goto(`${BASE}/`, { waitUntil: "load" });
  await page.waitForTimeout(400);

  const geometry = await page.evaluate(() => {
    const heading = document.querySelector('[data-testid="marketing-hero"] h1');
    const product = document.querySelector('[data-testid="hero-product"]');
    if (!heading || !product) return null;

    // Measure the real glyph boxes, not the block box, so a short last line
    // cannot mask an overlap.
    const range = document.createRange();
    range.selectNodeContents(heading);
    const glyphs = [...range.getClientRects()].map((rect) => ({
      right: rect.right,
      left: rect.left,
      top: rect.top,
      bottom: rect.bottom,
    }));
    const frame = product.getBoundingClientRect();
    const clipped = heading.scrollWidth - heading.clientWidth;
    return { glyphs, frame: { left: frame.left, top: frame.top, bottom: frame.bottom }, clipped };
  });

  if (!geometry) {
    failures.push(`${width}: hero heading or product frame missing`);
  } else {
    if (geometry.clipped > 1) failures.push(`${width}: headline clipped by ${geometry.clipped}px`);
    for (const glyph of geometry.glyphs) {
      const overlapsHorizontally = glyph.right > geometry.frame.left;
      const overlapsVertically =
        glyph.bottom > geometry.frame.top && glyph.top < geometry.frame.bottom;
      if (overlapsHorizontally && overlapsVertically) {
        failures.push(
          `${width}: headline glyph box (right ${Math.round(glyph.right)}) intersects product frame (left ${Math.round(geometry.frame.left)})`,
        );
        break;
      }
    }
  }
  await context.close();
}

// Every state chip must meet AA as text on its own background.
const context = await browser.newContext({ ...session, viewport: { width: 1440, height: 900 } });
const page = await context.newPage();

for (const route of ["/", "/evidence"]) {
  await page.goto(`${BASE}${route}`, { waitUntil: "load" });
  await page.waitForTimeout(400);

  const chips = await page.evaluate(() => {
    const backgroundOf = (node) => {
      let current = node;
      while (current) {
        const bg = getComputedStyle(current).backgroundColor;
        if (bg && bg !== "rgba(0, 0, 0, 0)" && bg !== "transparent") return bg;
        current = current.parentElement;
      }
      return "rgb(5, 5, 7)";
    };

    return [...document.querySelectorAll("span")]
      .filter((node) => {
        const previous = node.previousElementSibling;
        return (
          node.children.length === 0 &&
          previous?.getAttribute("aria-hidden") === "true" &&
          node.textContent?.trim()
        );
      })
      .map((node) => ({
        label: node.textContent.trim(),
        color: getComputedStyle(node).color,
        background: backgroundOf(node),
        fontSize: Number.parseFloat(getComputedStyle(node).fontSize),
      }));
  });

  for (const chip of chips) {
    const fg = parseRgb(chip.color);
    const bg = parseRgb(chip.background);
    if (!fg || !bg) continue;
    const ratio = contrast(fg, bg);
    if (ratio < 4.5) {
      failures.push(`${route}: chip "${chip.label}" contrast ${ratio.toFixed(2)}:1 (need 4.5)`);
    }
    if (chip.fontSize < 11) {
      failures.push(`${route}: chip "${chip.label}" font-size ${chip.fontSize}px (need 11)`);
    }
  }
  console.log(`${route}: checked ${chips.length} state chips`);
}

await context.close();

// A hydration mismatch means the served HTML and the client render disagree, so
// what a first-time reader sees is not what the code says. Docs pages are the
// ones that historically regressed, so they are checked alongside marketing.
const consoleContext = await browser.newContext({ ...session, viewport: { width: 1440, height: 900 } });
const consolePage = await consoleContext.newPage();
const noise = [/favicon/i, /Failed to load resource/i, /Clerk/i];

for (const route of ["/", "/evidence", "/download", "/docs", "/docs/concepts/what-is-pytxo"]) {
  const messages = [];
  const onConsole = (message) => {
    if (message.type() !== "error" && message.type() !== "warning") return;
    const text = message.text();
    if (noise.some((pattern) => pattern.test(text))) return;
    messages.push(text);
  };
  consolePage.on("console", onConsole);
  consolePage.on("pageerror", (error) => messages.push(error.message));
  await consolePage.goto(`${BASE}${route}`, { waitUntil: "load" });
  await consolePage.waitForTimeout(1200);
  consolePage.removeListener("console", onConsole);

  for (const message of messages) {
    failures.push(`${route}: console ${message.slice(0, 160)}`);
  }
  console.log(`${route}: ${messages.length} console error/warning(s)`);
}

await consoleContext.close();

// Public pages need stable search identity. Test the rendered head because
// streamed metadata can differ from the source-level metadata object.
const seoContext = await browser.newContext({ ...session, viewport: { width: 1280, height: 800 } });
const seoPage = await seoContext.newPage();
const publicRoutes = [
  "/",
  "/download",
  "/evidence",
  "/plans",
  "/docs",
  "/docs/concepts/what-is-pytxo",
];

for (const route of publicRoutes) {
  await seoPage.goto(`${BASE}${route}`, { waitUntil: "load" });
  const head = await seoPage.evaluate(() => ({
    title: document.title,
    description: document.querySelector('meta[name="description"]')?.getAttribute("content") ?? "",
    canonical: document.querySelector('link[rel="canonical"]')?.getAttribute("href") ?? "",
  }));
  const expectedCanonical = new URL(route, "https://pytxo.com").href;
  if (!head.title.trim()) failures.push(`${route}: missing title`);
  if (head.description.trim().length < 50) {
    failures.push(`${route}: description is missing or too short (${head.description.trim().length} chars)`);
  }
  if (!head.canonical || new URL(head.canonical, "https://pytxo.com").href !== expectedCanonical) {
    failures.push(`${route}: canonical ${head.canonical || "missing"} (expected ${expectedCanonical})`);
  }
}

await seoPage.goto(`${BASE}/`, { waitUntil: "load" });
const jsonLd = await seoPage.locator('script[type="application/ld+json"]').allTextContents();
const structuredTypes = jsonLd.flatMap((block) => {
  try {
    const parsed = JSON.parse(block);
    const records = Array.isArray(parsed?.["@graph"]) ? parsed["@graph"] : [parsed];
    return records.map((record) => record?.["@type"]).filter(Boolean);
  } catch {
    failures.push("/: invalid JSON-LD payload");
    return [];
  }
});
for (const type of ["WebSite", "SoftwareApplication"]) {
  if (!structuredTypes.includes(type)) failures.push(`/: JSON-LD missing ${type}`);
}

for (const route of ["/account", "/sign-in", "/sign-up"]) {
  await seoPage.goto(`${BASE}${route}`, { waitUntil: "load" });
  const robots =
    (await seoPage.locator('meta[name="robots"]').getAttribute("content"))?.toLowerCase() ?? "";
  if (!robots.includes("noindex")) failures.push(`${route}: private account surface is indexable`);
}

const robotsResponse = await seoPage.request.get(`${BASE}/robots.txt`);
const robotsText = await robotsResponse.text();
if (!robotsResponse.ok() || !robotsText.includes("https://pytxo.com/sitemap.xml")) {
  failures.push("/robots.txt: missing public sitemap declaration");
}

const sitemapResponse = await seoPage.request.get(`${BASE}/sitemap.xml`);
const sitemapText = await sitemapResponse.text();
if (!sitemapResponse.ok()) failures.push("/sitemap.xml: request failed");
for (const route of ["/download", "/evidence", "/plans", "/docs", "/docs/concepts/what-is-pytxo"]) {
  if (!sitemapText.includes(`<loc>https://pytxo.com${route}</loc>`)) {
    failures.push(`/sitemap.xml: missing ${route}`);
  }
}
for (const route of ["/account", "/sign-in", "/sign-up"]) {
  if (sitemapText.includes(`<loc>https://pytxo.com${route}</loc>`)) {
    failures.push(`/sitemap.xml: private route ${route} should not be listed`);
  }
}

await seoContext.close();
await browser.close();

if (failures.length > 0) {
  console.log("\nFAILURES:");
  for (const failure of [...new Set(failures)]) console.log(` - ${failure}`);
  process.exitCode = 1;
} else {
  console.log(
    "\nHero geometry clean at 1280/1440/1920; state chips pass AA at 11px or larger; no console errors; public metadata and structured data are complete; account routes are noindex.",
  );
}
