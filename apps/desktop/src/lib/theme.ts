export type DeckTheme = "void" | "light";
export type AccentPreset = "teal" | "custom";

export const THEME_STORAGE_KEY = "pytxo-deck-theme";
export const ACCENT_STORAGE_KEY = "pytxo-accent";
export const CUSTOM_ACCENT_STORAGE_KEY = "pytxo-accent-custom";
export const MATCH_SYSTEM_STORAGE_KEY = "pytxo-theme-match-system";
/** @deprecated Legacy boolean completion flag, superseded by `ONBOARDING_VERSION_KEY`. */
export const SETUP_STORAGE_KEY = "pytxo-deck-setup-v1";

/**
 * Bump this whenever onboarding content changes meaningfully enough that
 * existing users should see it again (new steps, new defaults to confirm,
 * etc). Comparing against a stored version — rather than a boolean — means
 * upgrading users see the refreshed flow exactly once, without re-showing it
 * on every subsequent release.
 */
export const ONBOARDING_VERSION = "1.1.0";
export const ONBOARDING_VERSION_KEY = "pytxo-desktop-onboarding-version";

export const DECK_THEMES: { id: DeckTheme; label: string; hint: string }[] = [
  { id: "void", label: "Void", hint: "Canonical dark" },
  { id: "light", label: "Light", hint: "Bright workspace" },
];

export const ACCENT_PRESETS: { id: Exclude<AccentPreset, "custom">; label: string }[] = [
  { id: "teal", label: "Teal" },
];

const DEFAULT_CUSTOM_ACCENT = "#2dd4bf";

function canonicalizeTheme(raw: string | null): DeckTheme {
  if (raw === "light") return "light";
  return "void";
}

export function loadTheme(): DeckTheme {
  if (typeof localStorage === "undefined") return "void";
  return canonicalizeTheme(localStorage.getItem(THEME_STORAGE_KEY));
}

export function loadAccent(): AccentPreset {
  if (typeof localStorage === "undefined") return "teal";
  const raw = localStorage.getItem(ACCENT_STORAGE_KEY);
  if (raw === "custom") return "custom";
  return "teal";
}

export function loadCustomAccent(): string {
  if (typeof localStorage === "undefined") return DEFAULT_CUSTOM_ACCENT;
  const raw = localStorage.getItem(CUSTOM_ACCENT_STORAGE_KEY);
  if (raw && /^#[0-9a-fA-F]{6}$/.test(raw)) return raw;
  return DEFAULT_CUSTOM_ACCENT;
}

export function loadMatchSystem(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(MATCH_SYSTEM_STORAGE_KEY) === "true";
}

export function applyDeckTheme(theme: DeckTheme) {
  const root = document.documentElement;
  const next = canonicalizeTheme(theme);
  root.setAttribute("data-chroma-theme", next);
  if (typeof localStorage !== "undefined") localStorage.setItem(THEME_STORAGE_KEY, next);
  if (next === "light") {
    root.classList.remove("dark");
    root.style.colorScheme = "light";
  } else {
    root.classList.add("dark");
    root.style.colorScheme = "dark";
  }
}

export function applyAccent(accent: AccentPreset, customHex?: string) {
  const root = document.documentElement;
  const next: AccentPreset = accent === "custom" ? "custom" : "teal";
  root.setAttribute("data-pytxo-accent", next);
  if (typeof localStorage !== "undefined") localStorage.setItem(ACCENT_STORAGE_KEY, next);

  if (next === "custom") {
    const hex = customHex ?? loadCustomAccent();
    root.style.setProperty("--pytxo-accent", hex);
    if (typeof localStorage !== "undefined") localStorage.setItem(CUSTOM_ACCENT_STORAGE_KEY, hex);
  } else {
    root.style.removeProperty("--pytxo-accent");
  }
}

export function applyMatchSystem(enabled: boolean) {
  if (typeof localStorage !== "undefined") {
    localStorage.setItem(MATCH_SYSTEM_STORAGE_KEY, enabled ? "true" : "false");
  }
  if (!enabled) return loadTheme();
  const prefersLight =
    typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: light)").matches;
  const next: DeckTheme = prefersLight ? "light" : "void";
  applyDeckTheme(next);
  return next;
}

let systemThemeMql: MediaQueryList | null = null;
let systemThemeHandler: ((e: MediaQueryListEvent) => void) | null = null;

export function syncSystemThemeListener(enabled: boolean) {
  if (typeof window === "undefined") return;
  if (systemThemeMql && systemThemeHandler) {
    systemThemeMql.removeEventListener("change", systemThemeHandler);
    systemThemeMql = null;
    systemThemeHandler = null;
  }
  if (!enabled) return;
  systemThemeMql = window.matchMedia("(prefers-color-scheme: light)");
  systemThemeHandler = (e) => {
    if (!loadMatchSystem()) return;
    applyDeckTheme(e.matches ? "light" : "void");
  };
  systemThemeMql.addEventListener("change", systemThemeHandler);
}

export function initThemeChrome() {
  const matchSystem = loadMatchSystem();
  if (matchSystem) {
    applyMatchSystem(true);
  } else {
    applyDeckTheme(loadTheme());
  }
  applyAccent(loadAccent());
  syncSystemThemeListener(matchSystem);
}

export function isSetupComplete(): boolean {
  return localStorage.getItem(ONBOARDING_VERSION_KEY) === ONBOARDING_VERSION;
}

export function markSetupComplete() {
  localStorage.setItem(ONBOARDING_VERSION_KEY, ONBOARDING_VERSION);
  localStorage.setItem(SETUP_STORAGE_KEY, "complete");
}

/** Used by Settings → "Run onboarding again". */
export function resetOnboarding() {
  localStorage.removeItem(ONBOARDING_VERSION_KEY);
}

export function terminalThemeFor(deckTheme: DeckTheme) {
  if (deckTheme === "light") {
    return { background: "#f4f6f8", foreground: "#0f1419" };
  }
  return { background: "#07080b", foreground: "#e8eaed" };
}

export function isLightDeck(theme: DeckTheme) {
  return theme === "light";
}
