import type { AdeCliStatusDto } from "./types";

/** CLIs Desktop Beta can dispatch; mirrors `DESKTOP_BETA_ADES` in Core. */
export const DESKTOP_BETA_ADES = ["codex", "claude", "cursor", "opencode", "agy"] as const;
export const DESKTOP_BETA_MAX_WORKERS = 8;

const ADE_NAMES: Record<string, string> = {
  codex: "OpenAI Codex", claude: "Claude Code", cursor: "Cursor Agent", opencode: "OpenCode", agy: "Antigravity",
  gemini: "Gemini CLI", copilot: "GitHub Copilot CLI", aider: "Aider", grok: "Grok Build", droid: "Factory Droid",
  cline: "Cline CLI", goose: "Goose", qwen: "Qwen Code", kimi: "Kimi Code CLI",
};

/** Registry display name for a CLI id, when Pytxo knows it. */
export function adeDisplayName(id: string | null | undefined): string | null {
  return id ? ADE_NAMES[id] ?? null : null;
}

/**
 * Admitted to Desktop runs but not yet seen completing a live end-to-end run
 * on a released build (10 Oct 2026: Antigravity's headless mode denied its own
 * tool permission; OpenCode's default model could not use tools).
 */
export const UNVERIFIED_LIVE_ADES = ["opencode", "agy"] as const;

export function isUnverifiedLiveAde(id: string): boolean {
  return (UNVERIFIED_LIVE_ADES as readonly string[]).includes(id);
}

export function isBetaAde(id: string): boolean {
  return (DESKTOP_BETA_ADES as readonly string[]).includes(id);
}

/** Beta CLIs in their canonical order (Codex first); other CLIs are dropped. */
export function betaAdesInOrder<T extends { id: string }>(clis: T[]): T[] {
  return DESKTOP_BETA_ADES.flatMap((id) => clis.filter((cli) => cli.id === id));
}

/**
 * Runnable does not always mean that Pytxo verified an account session. Some
 * harnesses deliberately keep authentication opaque and validate it only when
 * their process starts. Keep that distinct from a confirmed signed-in state.
 */
export function isAdeRunnable(cli: AdeCliStatusDto): boolean {
  return cli.installed && ["signed_in", "vendor_managed", "not_applicable"].includes(cli.auth_state);
}

export function isAdeSessionConfirmed(cli: AdeCliStatusDto): boolean {
  return cli.installed && ["signed_in", "not_applicable"].includes(cli.auth_state);
}

export function adeAvailabilityLabel(cli: AdeCliStatusDto): string {
  if (!cli.installed || cli.auth_state === "not_installed") return "not installed";
  // Name a non-default account so Work says which sign-in will run.
  const account = cli.active_account && cli.active_account !== "default" ? ` · account ${cli.active_account}` : "";
  if (cli.auth_state === "signed_out") return `sign-in required${account}`;
  if (cli.auth_state === "unknown") return `sign-in could not be verified${account}`;
  if (cli.auth_state === "detected_only") return "detected · write mode not mapped";
  if (cli.auth_state === "vendor_managed") return "available · vendor-managed authentication";
  return `ready${account}`;
}
