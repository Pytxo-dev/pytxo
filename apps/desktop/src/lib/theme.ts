export type DeckTheme = "void" | "light" | "terminal" | "nebula";

export const THEME_STORAGE_KEY = "pytxo-deck-theme";
export const SETUP_STORAGE_KEY = "pytxo-deck-setup-v1";

export const DECK_THEMES: { id: DeckTheme; label: string }[] = [
  { id: "void", label: "Void" },
  { id: "light", label: "Light" },
  { id: "terminal", label: "Terminal" },
  { id: "nebula", label: "Nebula" },
];

export function loadTheme(): DeckTheme {
  const raw = localStorage.getItem(THEME_STORAGE_KEY);
  if (raw === "light" || raw === "terminal" || raw === "nebula" || raw === "void") {
    return raw;
  }
  if (raw === "dark") return "void";
  return "void";
}

export function applyDeckTheme(theme: DeckTheme) {
  const root = document.documentElement;
  root.setAttribute("data-chroma-theme", theme);
  localStorage.setItem(THEME_STORAGE_KEY, theme);
  if (theme === "light") {
    root.classList.remove("dark");
    root.style.colorScheme = "light";
  } else {
    root.classList.add("dark");
    root.style.colorScheme = "dark";
  }
}

export function isSetupComplete(): boolean {
  return localStorage.getItem(SETUP_STORAGE_KEY) === "complete";
}

export function markSetupComplete() {
  localStorage.setItem(SETUP_STORAGE_KEY, "complete");
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
