/**
 * Persisted display preferences for Pytxo Desktop.
 *
 * Scale maps to the real Tauri webview zoom factor (`setZoom`) so text,
 * icons, and hit targets shrink/grow together instead of a CSS-only
 * approximation. Density toggles a data attribute that compact/comfortable
 * component styles read from, independent of scale. Default scale is 100%.
 *
 * OS DPI (125%/150% Windows) is handled by the webview itself — we do **not**
 * multiply user scale by `scaleFactor` (that double-zooms). On boot and on
 * monitor DPI changes we re-apply the user zoom so chrome stays crisp.
 */
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { getCurrentWindow } from "@tauri-apps/api/window";

export type UiScale = 0.85 | 0.9 | 1 | 1.1;
export type UiDensity = "compact" | "comfortable";

const SCALE_KEY = "pytxo-desktop-scale-v1";
const DENSITY_KEY = "pytxo-desktop-density-v1";
const MOTION_KEY = "pytxo-desktop-reduced-motion-v1";
const VALID_SCALES: readonly number[] = [0.85, 0.9, 1, 1.1];

export const UI_SCALES: { value: UiScale; label: string }[] = [
  { value: 0.85, label: "85%" },
  { value: 0.9, label: "90%" },
  { value: 1, label: "100%" },
  { value: 1.1, label: "110%" },
];

function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function readScale(): UiScale {
  if (typeof localStorage === "undefined") return 1;
  const stored = localStorage.getItem(SCALE_KEY);
  if (stored === null) return 1;
  const raw = Number(stored);
  if (VALID_SCALES.includes(raw)) return raw as UiScale;
  localStorage.setItem(SCALE_KEY, "1");
  return 1;
}

function readDensity(): UiDensity {
  if (typeof localStorage === "undefined") return "compact";
  return localStorage.getItem(DENSITY_KEY) === "comfortable" ? "comfortable" : "compact";
}

/** Explicit in-app override; `false` still respects the OS `prefers-reduced-motion` media query. */
function readReducedMotion(): boolean {
  if (typeof localStorage === "undefined") return false;
  return localStorage.getItem(MOTION_KEY) === "true";
}

export const uiPrefs = $state<{ scale: UiScale; density: UiDensity; reducedMotion: boolean }>({
  scale: readScale(),
  density: readDensity(),
  reducedMotion: readReducedMotion(),
});

/** Apply zoom via the real Tauri webview API; CSS `zoom` fallback for browser/Storybook previews. */
export async function applyScale(scale: UiScale = uiPrefs.scale) {
  if (isTauriRuntime()) {
    try {
      await getCurrentWebview().setZoom(scale);
      document.documentElement.style.removeProperty("zoom");
      return;
    } catch {
      /* older webview without setZoom support — fall through to CSS zoom */
    }
  }
  document.documentElement.style.setProperty("zoom", String(scale));
}

export function applyDensity(density: UiDensity = uiPrefs.density) {
  document.documentElement.setAttribute("data-ui-density", density);
}

export async function setUiScale(scale: UiScale) {
  uiPrefs.scale = scale;
  if (typeof localStorage !== "undefined") localStorage.setItem(SCALE_KEY, String(scale));
  await applyScale(scale);
}

export function setUiDensity(density: UiDensity) {
  uiPrefs.density = density;
  if (typeof localStorage !== "undefined") localStorage.setItem(DENSITY_KEY, density);
  applyDensity(density);
}

export function applyReducedMotion(reducedMotion: boolean = uiPrefs.reducedMotion) {
  document.documentElement.toggleAttribute("data-force-reduced-motion", reducedMotion);
}

export function setReducedMotion(reducedMotion: boolean) {
  uiPrefs.reducedMotion = reducedMotion;
  if (typeof localStorage !== "undefined") localStorage.setItem(MOTION_KEY, String(reducedMotion));
  applyReducedMotion(reducedMotion);
}

let scaleUnlisten: (() => void) | null = null;

/** Call once on app boot, after onboarding has chosen (or defaulted) a scale. */
export function initUiPrefs() {
  applyDensity(uiPrefs.density);
  applyReducedMotion(uiPrefs.reducedMotion);
  void applyScale(uiPrefs.scale);

  if (!isTauriRuntime()) return;
  void (async () => {
    try {
      // Touch OS factor so we notice monitor DPI; zoom stays user-scale only.
      await getCurrentWindow().scaleFactor();
      if (scaleUnlisten) {
        scaleUnlisten();
        scaleUnlisten = null;
      }
      scaleUnlisten = await getCurrentWindow().onScaleChanged(() => {
        void applyScale(uiPrefs.scale);
      });
    } catch {
      /* Preview / missing window API */
    }
  })();
}
