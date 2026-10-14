import { auth } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";
import { linkUrl, NO_STORE } from "@/lib/link";

export const dynamic = "force-dynamic";

const RECEIPT_ID = /^r_[0-9A-Za-z]{12}$/;

/** Calls Link as the signed-in website user; a NextResponse return is an error to send as-is. */
async function asUser(path: string, init: RequestInit = {}): Promise<Response | NextResponse> {
  if (!SITE_AUTH_BACKEND_ENABLED) {
    return NextResponse.json({ error: "account_unavailable" }, { status: 503, headers: NO_STORE });
  }
  const { userId, getToken } = await auth();
  const token = userId ? await getToken() : null;
  if (!token) return NextResponse.json({ error: "unauthorized" }, { status: 401, headers: NO_STORE });
  const target = linkUrl(path);
  if (!target) return NextResponse.json({ error: "account_unavailable" }, { status: 503, headers: NO_STORE });
  try {
    return await fetch(target, {
      ...init,
      headers: { Authorization: `Bearer ${token}` },
      cache: "no-store",
      signal: AbortSignal.timeout(5_000),
    });
  } catch {
    return NextResponse.json({ error: "link_unreachable" }, { status: 502, headers: NO_STORE });
  }
}

/** The signed-in user's shared receipts (account perk: receipt history). */
export async function GET() {
  const upstream = await asUser("/v1/receipts");
  if (upstream instanceof NextResponse) return upstream;
  if (!upstream.ok) return NextResponse.json({ error: "link_unreachable" }, { status: 502, headers: NO_STORE });
  return NextResponse.json(await upstream.json(), { headers: NO_STORE });
}

/** Deletes one of the signed-in user's shared receipts (`?id=r_…`). */
export async function DELETE(req: NextRequest) {
  const id = req.nextUrl.searchParams.get("id") ?? "";
  if (!RECEIPT_ID.test(id)) return NextResponse.json({ error: "invalid_id" }, { status: 400, headers: NO_STORE });
  const upstream = await asUser(`/v1/receipts/${id}`, { method: "DELETE" });
  if (upstream instanceof NextResponse) return upstream;
  return new NextResponse(null, { status: upstream.status === 204 ? 204 : upstream.status === 404 ? 404 : 502, headers: NO_STORE });
}
