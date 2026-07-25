export type DeckTheme = "void" | "light" | "terminal" | "nebula";
export type AccentPreset = "spectrum" | "teal" | "violet" | "gold";

export const THEME_STORAGE_KEY = "pytxo-deck-theme";
export const ACCENT_STORAGE_KEY = "pytxo-accent";
/** @deprecated Legacy boolean completion flag, superseded by `ONBOARDING_VERSION_KEY`. */
export const SETUP_STORAGE_KEY = "pytxo-deck-setup-v1";

/**
 * Bump this whenever onboarding content changes meaningfully enough that
 * existing users should see it again (new steps, new defaults to confirm,
 * etc). Comparing against a stored version — rather than a boolean — means
 * upgrading users see the refreshed flow exactly once, without re-showing it
 * on every subsequent release.
 */
export const ONBOARDING_VERSION = "0.7.0";
export const ONBOARDING_VERSION_KEY = "pytxo-desktop-onboarding-version";

/** Void first; Nebula demoted as vivid optional skin. */
export const DECK_THEMES: { id: DeckTheme; label: string }[] = [
  { id: "void", label: "Void" },
  { id: "light", label: "Light" },
  { id: "terminal", label: "Terminal" },
  { id: "nebula", label: "Nebula" },
];

export const ACCENT_PRESETS: { id: AccentPreset; label: string }[] = [
  { id: "spectrum", label: "Spectrum" },
  { id: "teal", label: "Teal" },
  { id: "violet", label: "Violet" },
  { id: "gold", label: "Gold" },
];

export function loadTheme(): DeckTheme {
  if (typeof localStorage === "undefined") return "void";
  const raw = localStorage.getItem(THEME_STORAGE_KEY);
  if (raw === "light" || raw === "terminal" || raw === "nebula" || raw === "void") {
    return raw;
  }
  if (raw === "dark") return "void";
  return "void";
}

export function loadAccent(): AccentPreset {
  if (typeof localStorage === "undefined") return "spectrum";
  const raw = localStorage.getItem(ACCENT_STORAGE_KEY);
  if (raw === "spectrum" || raw === "teal" || raw === "violet" || raw === "gold") return raw;
  return "spectrum";
}

export function applyDeckTheme(theme: DeckTheme) {
  const root = document.documentElement;
  root.setAttribute("data-chroma-theme", theme);
  if (typeof localStorage !== "undefined") localStorage.setItem(THEME_STORAGE_KEY, theme);
  if (theme === "light") {
    root.classList.remove("dark");
    root.style.colorScheme = "light";
  } else {
    root.classList.add("dark");
    root.style.colorScheme = "dark";
  }
}

export function applyAccent(accent: AccentPreset) {
  const root = document.documentElement;
  root.setAttribute("data-pytxo-accent", accent);
  if (typeof localStorage !== "undefined") localStorage.setItem(ACCENT_STORAGE_KEY, accent);
}

export function initThemeChrome() {
  applyDeckTheme(loadTheme());
  applyAccent(loadAccent());
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
  if (deckTheme === "terminal") {
    return { background: "#001a0a", foreground: "#4ade80" };
  }
  return { background: "#020205", foreground: "#e8eaed" };
}

export function isLightDeck(theme: DeckTheme) {
  return theme === "light";
}
