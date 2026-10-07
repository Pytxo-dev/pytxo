/** Modifier key label for shortcut hints on the current platform. */
export const MOD_KEY =
  typeof navigator !== "undefined" && /Mac|iPhone|iPod|iPad/.test(navigator.userAgent) ? "⌘" : "Ctrl";
