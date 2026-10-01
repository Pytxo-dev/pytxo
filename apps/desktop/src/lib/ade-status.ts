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

export function isBetaAde(id: string): boolean {
  return (DESKTOP_BETA_ADES as readonly string[]).includes(id);
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
  if (cli.auth_state === "signed_out") return "sign-in required";
  if (cli.auth_state === "unknown") return "sign-in could not be verified";
  if (cli.auth_state === "detected_only") return "detected · write mode not mapped";
  if (cli.auth_state === "vendor_managed") return "available · vendor-managed authentication";
  return "ready";
}
