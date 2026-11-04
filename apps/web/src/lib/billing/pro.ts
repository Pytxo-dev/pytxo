/**
 * The hosted account's Pro tier. Shared by server and client components, so
 * it holds limits only (no secrets).
 *
 * Checkout is closed to new buyers: Pytxo Pro is the GitHub Action on private
 * repositories (`PRO_LAUNCH` in lib/site.ts), not this hosted tier, and it is
 * not on sale yet. Existing subscribers keep Manage billing.
 */
export const PRO_CHECKOUT_OPEN = false;

/** Mirrors FREE_RECEIPT_LIMIT in services/pytxo-link/src/account.rs. */
export const FREE_RECEIPT_LIMIT = 25;

/** Link tiers that count as Pro (Max and Ultra are retired plan names). */
export function isProTier(tier: string | null | undefined): boolean {
  return tier === "pro" || tier === "max" || tier === "ultra";
}
