import type { Page } from "@playwright/test";
import {
  ONBOARDING_VERSION,
  ONBOARDING_VERSION_KEY,
  SETUP_STORAGE_KEY,
} from "../src/lib/theme";

export { SETUP_STORAGE_KEY } from "../src/lib/theme";

export const SCALE_STORAGE_KEY = "pytxo-desktop-scale-v1";

/** Bypass the versioned onboarding gate for shell-level browser-preview tests. */
export async function completeOnboarding(page: Page, extra?: Record<string, string>) {
  await page.addInitScript(
    ({ versionKey, version, setupKey, extra }) => {
      localStorage.setItem(setupKey, "complete");
      localStorage.setItem(versionKey, version);
      for (const [key, value] of Object.entries(extra)) localStorage.setItem(key, value);
    },
    {
      versionKey: ONBOARDING_VERSION_KEY,
      version: ONBOARDING_VERSION,
      setupKey: SETUP_STORAGE_KEY,
      extra: extra ?? {},
    },
  );
}

export async function clearOnboarding(page: Page) {
  await page.addInitScript(
    ({ versionKey, setupKey }) => {
      localStorage.removeItem(versionKey);
      localStorage.removeItem(setupKey);
      localStorage.removeItem("pytxo-deck-tabs-v1");
    },
    { versionKey: ONBOARDING_VERSION_KEY, setupKey: SETUP_STORAGE_KEY },
  );
}

export async function openAppearance(page: Page) {
  await page.goto("/#/settings");
  await page.getByRole("button", { name: "Account & billing" }).click();
  await page.getByRole("button", { name: "Appearance" }).click();
}

export async function setBrowserPreviewScale(page: Page, label: "90%" | "100%" | "110%") {
  await page.getByRole("button", { name: label, exact: true }).click();
}

export async function rootOverflow(page: Page) {
  return page.evaluate(() => ({
    horizontal: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    vertical: document.documentElement.scrollHeight - document.documentElement.clientHeight,
  }));
}

export async function expectRootZoom(page: Page, scale: number) {
  const zoom = await page.evaluate(() => getComputedStyle(document.documentElement).zoom);
  if (zoom !== String(scale)) {
    throw new Error(`Expected root CSS zoom ${scale}, received ${zoom}.`);
  }
}
