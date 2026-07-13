import crypto from "node:crypto";

import type { CheckoutPlan } from "./mbcz-checkout";

const PRICE_TO_TIER: Record<string, CheckoutPlan> = {};

function loadPriceMap() {
  const entries: [string | undefined, CheckoutPlan][] = [
    [process.env.PADDLE_PRICE_PRO, "pro"],
    [process.env.PADDLE_PRICE_MAX, "max"],
    [process.env.PADDLE_PRICE_ULTRA, "ultra"],
  ];
  for (const [priceId, tier] of entries) {
    if (priceId) PRICE_TO_TIER[priceId] = tier;
  }
}

export function tierFromPriceId(priceId: string | undefined): CheckoutPlan | null {
  if (!priceId) return null;
  if (Object.keys(PRICE_TO_TIER).length === 0) loadPriceMap();
  return PRICE_TO_TIER[priceId] ?? null;
}

export function tierFromCustomData(tier: string | undefined): CheckoutPlan | null {
  if (!tier) return null;
  const normalized = tier.toLowerCase();
  if (normalized === "pro" || normalized === "pro_cloud") return "pro";
  if (normalized === "max" || normalized === "max_swarm") return "max";
  if (normalized === "ultra") return "ultra";
  return null;
}

export function verifyPaddleSignature(
  rawBody: string,
  signatureHeader: string | null,
  secret: string,
): boolean {
  if (!signatureHeader || !secret) return false;
  const parts = Object.fromEntries(
    signatureHeader.split(";").map((p) => {
      const [k, v] = p.split("=");
      return [k.trim(), v?.trim() ?? ""];
    }),
  );
  const ts = parts.ts;
  const h1 = parts.h1;
  if (!ts || !h1) return false;
  const payload = `${ts}:${rawBody}`;
  const expected = crypto.createHmac("sha256", secret).update(payload).digest("hex");
  try {
    return crypto.timingSafeEqual(Buffer.from(h1), Buffer.from(expected));
  } catch {
    return false;
  }
}

export type PaddleEvent = {
  event_type?: string;
  data?: {
    custom_data?: { user_id?: string; tier?: string; org_id?: string };
    items?: Array<{ price?: { id?: string } }>;
    status?: string;
  };
};

export function extractUserId(event: PaddleEvent): string | null {
  return event.data?.custom_data?.user_id ?? null;
}

export function extractOrgId(event: PaddleEvent): string | null {
  const orgId = event.data?.custom_data?.org_id;
  return orgId && orgId.trim().length > 0 ? orgId.trim() : null;
}

export function extractTier(event: PaddleEvent): CheckoutPlan | null {
  const fromCustom = tierFromCustomData(event.data?.custom_data?.tier);
  if (fromCustom) return fromCustom;
  const priceId = event.data?.items?.[0]?.price?.id;
  return tierFromPriceId(priceId);
}
