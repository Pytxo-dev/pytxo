import { NextRequest, NextResponse } from "next/server";

import { provisionLinkEntitlement } from "@/lib/billing/link-admin";
import {
  extractTier,
  extractUserId,
  verifyPaddleSignature,
  type PaddleEvent,
} from "@/lib/billing/paddle";

export const runtime = "nodejs";

export async function POST(req: NextRequest) {
  const secret = process.env.PADDLE_WEBHOOK_SECRET;
  if (!secret) {
    return NextResponse.json({ error: "Webhook not configured" }, { status: 500 });
  }

  const rawBody = await req.text();
  const signature = req.headers.get("paddle-signature");
  if (!verifyPaddleSignature(rawBody, signature, secret)) {
    return NextResponse.json({ error: "Invalid signature" }, { status: 401 });
  }

  const event = JSON.parse(rawBody) as PaddleEvent;
  const eventType = event.event_type ?? "";

  if (
    eventType !== "subscription.created" &&
    eventType !== "subscription.updated" &&
    eventType !== "subscription.canceled"
  ) {
    return NextResponse.json({ ok: true, skipped: true });
  }

  const userId = extractUserId(event);
  if (!userId) {
    console.warn("[billing-webhook] skip provisioning — missing custom_data user_id");
    return NextResponse.json({ ok: true, skipped: true });
  }

  if (eventType === "subscription.canceled") {
    await provisionLinkEntitlement({ userId, tier: "core" });
    return NextResponse.json({ ok: true });
  }

  const tier = extractTier(event);
  if (!tier) {
    console.warn("[billing-webhook] skip provisioning — unknown tier");
    return NextResponse.json({ ok: true, skipped: true });
  }

  await provisionLinkEntitlement({ userId, tier });
  return NextResponse.json({ ok: true });
}
