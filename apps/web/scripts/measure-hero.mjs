// Temporary geometry probe for the hero headline / product-frame collision.
// Run against a dev server: node scripts/measure-hero.mjs http://127.0.0.1:3111
import { chromium } from "playwright";

const base = process.argv[2] ?? "http://127.0.0.1:3111";
const widths = [1280, 1440, 1920];

const browser = await chromium.launch();
for (const width of widths) {
  const page = await browser.newPage({ viewport: { width, height: 900 } });
  await page.goto(base, { waitUntil: "networkidle" });
  const result = await page.evaluate(() => {
    const h1 = document.querySelector("h1");
    const frame = document.querySelector('[data-testid="hero-product"]');
    const box = (el) => {
      const r = el.getBoundingClientRect();
      return { x: Math.round(r.x), right: Math.round(r.right), y: Math.round(r.y), bottom: Math.round(r.bottom), w: Math.round(r.width), h: Math.round(r.height) };
    };
    const style = getComputedStyle(h1);
    const lines = h1.getBoundingClientRect().height / Number.parseFloat(style.lineHeight);
    return {
      h1: box(h1),
      frame: frame ? box(frame.closest("div.relative.overflow-hidden") ?? frame) : null,
      inner: frame ? box(frame) : null,
      fontSize: style.fontSize,
      lines: Number(lines.toFixed(2)),
      overflow: h1.scrollWidth - h1.clientWidth,
    };
  });
  const gap = result.frame ? result.frame.x - result.h1.right : null;
  console.log(
    `${width}px  font=${result.fontSize} lines=${result.lines} h1Overflow=${result.overflow}\n` +
      `        h1   x=${result.h1.x} right=${result.h1.right} w=${result.h1.w}\n` +
      `        frame x=${result.frame?.x} w=${result.frame?.w} y=${result.frame?.y} bottom=${result.frame?.bottom}\n` +
      `        gap(h1.right -> frame.x) = ${gap}px  ${gap !== null && gap >= 0 ? "OK" : "COLLISION"}`,
  );
  await page.close();
}
await browser.close();
