import { expect, test } from "@playwright/test";

test.use({ javaScriptEnabled: false });

test("every sitemap page serves its own crawlable identity without JavaScript", async ({ page, request }) => {
  const sitemap = await request.get("/sitemap.xml");
  expect(sitemap.status()).toBe(200);
  const locations = [...(await sitemap.text()).matchAll(/<loc>([^<]+)<\/loc>/g)].map((match) => match[1]);
  expect(locations.length).toBeGreaterThan(4);
  expect(new Set(locations).size).toBe(locations.length);
  expect(locations).toContain("https://pytxo.com/evidence");

  for (const location of locations) {
    const url = new URL(location);
    expect(url.origin).toBe("https://pytxo.com");
    const response = await page.goto(url.pathname);
    expect(response?.status(), url.pathname).toBe(200);
    const canonical = page.locator('link[rel="canonical"]');
    await expect(canonical).toHaveCount(1);
    expect(new URL((await canonical.getAttribute("href"))!).href).toBe(url.href);
    const title = await page.title();
    expect(title.length).toBeGreaterThan(5);
    await expect(page.locator('meta[property="og:title"]')).toHaveAttribute("content", title);
    await expect(page.locator('meta[name="twitter:title"]')).toHaveAttribute("content", title);
    const description = await page.locator('meta[name="description"]').getAttribute("content");
    expect(description?.length).toBeGreaterThan(10);
    await expect(page.locator('meta[property="og:description"]')).toHaveAttribute("content", description!);
    expect(new URL((await page.locator('meta[property="og:url"]').getAttribute("content"))!).href).toBe(url.href);
    await expect(page.locator("h1"), url.pathname).toHaveCount(1);
    const robots = await page.locator('meta[name="robots"]').getAttribute("content", { timeout: 100 }).catch(() => "");
    expect(robots ?? "").not.toContain("noindex");
  }
});

test("tracking parameters do not change canonical or share URLs", async ({ page }) => {
  await page.goto("/evidence?utm_source=acceptance");
  await expect(page.locator('link[rel="canonical"]')).toHaveAttribute("href", "https://pytxo.com/evidence");
  await expect(page.locator('meta[property="og:url"]')).toHaveAttribute("content", "https://pytxo.com/evidence");
});

test("robots exposes the sitemap and missing pages return a nonindexable 404", async ({ page, request }) => {
  const response = await request.get("/robots.txt");
  expect(response.status()).toBe(200);
  const robots = await response.text();
  expect(robots).toContain("User-Agent: *");
  expect(robots).toContain("Allow: /");
  expect(robots).toContain("Sitemap: https://pytxo.com/sitemap.xml");
  const missing = await page.goto("/metadata-acceptance-missing-page");
  expect(missing?.status()).toBe(404);
  await expect(page.locator('meta[name="robots"]')).toHaveAttribute("content", /noindex/);
});

test("social image is available with the declared dimensions and type", async ({ page, request }) => {
  await page.goto("/evidence");
  const image = await page.locator('meta[property="og:image"]').getAttribute("content");
  const url = new URL(image!);
  expect(url.origin).toBe("https://pytxo.com");
  const response = await request.get(url.pathname);
  expect(response.status()).toBe(200);
  expect(response.headers()["content-type"]).toContain("image/png");
  const bytes = await response.body();
  expect(bytes.subarray(0, 8).toString("hex")).toBe("89504e470d0a1a0a");
  const width = Number(await page.locator('meta[property="og:image:width"]').getAttribute("content"));
  const height = Number(await page.locator('meta[property="og:image:height"]').getAttribute("content"));
  expect(bytes.readUInt32BE(16)).toBe(width);
  expect(bytes.readUInt32BE(20)).toBe(height);
  await expect(page.locator('meta[name="twitter:image"]')).toHaveAttribute("content", image!);
  await expect(page.locator('meta[name="twitter:card"]')).toHaveAttribute("content", "summary_large_image");
});
