import type { AdeCliStatusDto } from "./types";

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
