/**
 * Pytxo Pro: the one paid plan. Shared by server and client components, so
 * it holds copy and limits only (no secrets). The price is set on the Dodo
 * product; keep PRO_PRICE in step with it.
 */
export const PRO_PRICE = "$12";
export const PRO_INTERVAL = "month";

/** Mirrors FREE_RECEIPT_LIMIT in services/pytxo-link/src/account.rs. */
export const FREE_RECEIPT_LIMIT = 25;

export const PRO_PERKS = [
  ["Private receipts", "Share a receipt that only you can open, signed in."],
  ["Unlimited receipts", `Keep every shared receipt. Free accounts keep ${FREE_RECEIPT_LIMIT}.`],
  ["Private Pybench", "Save Pybench results to your account without publishing them to the board."],
] as const;

/** Link tiers that count as Pro (Max and Ultra are retired plan names). */
export function isProTier(tier: string | null | undefined): boolean {
  return tier === "pro" || tier === "max" || tier === "ultra";
}
