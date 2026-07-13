import type { CheckoutPlan } from "./mbcz-checkout";

export type LinkTier = CheckoutPlan | "core";

export async function provisionLinkEntitlement(input: {
  userId: string;
  tier: LinkTier;
  clerkUserId?: string;
  orgId?: string;
}): Promise<void> {
  const base = process.env.LINK_ADMIN_URL ?? "https://link.pytxo.com";
  const adminKey = process.env.LINK_ADMIN_KEY;
  if (!adminKey) {
    throw new Error("LINK_ADMIN_KEY is not configured");
  }

  const cloudEnabled = input.tier === "max" || input.tier === "ultra";
  const maxAgents =
    input.tier === "ultra"
      ? 256
      : input.tier === "max"
        ? 128
        : input.tier === "pro"
          ? 64
          : 3;

  const res = await fetch(`${base}/v1/admin/entitlements/${encodeURIComponent(input.userId)}`, {
    method: "PUT",
    headers: {
      Authorization: `Bearer ${adminKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      tier: input.tier,
      clerk_user_id: input.clerkUserId,
      org_id: input.orgId,
      max_agents: maxAgents,
      cloud_enabled: cloudEnabled,
    }),
  });

  if (!res.ok) {
    throw new Error(`Link admin provision failed: ${res.status} ${await res.text()}`);
  }
}
