import { auth } from "@clerk/nextjs/server";
import { NextResponse } from "next/server";

import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";
import { linkUrl, NO_STORE } from "@/lib/link";

export const dynamic = "force-dynamic";

/** The signed-in user's Pybench submissions, public and private (Pro). */
export async function GET() {
  if (!SITE_AUTH_BACKEND_ENABLED) {
    return NextResponse.json({ error: "account_unavailable" }, { status: 503, headers: NO_STORE });
  }
  const { userId, getToken } = await auth();
  const token = userId ? await getToken() : null;
  if (!token) return NextResponse.json({ error: "unauthorized" }, { status: 401, headers: NO_STORE });
  const target = linkUrl("/v1/bench/mine");
  if (!target) return NextResponse.json({ error: "account_unavailable" }, { status: 503, headers: NO_STORE });
  try {
    const upstream = await fetch(target, {
      headers: { Authorization: `Bearer ${token}` },
      cache: "no-store",
      signal: AbortSignal.timeout(5_000),
    });
    if (!upstream.ok) return NextResponse.json({ error: "link_unreachable" }, { status: 502, headers: NO_STORE });
    return NextResponse.json(await upstream.json(), { headers: NO_STORE });
  } catch {
    return NextResponse.json({ error: "link_unreachable" }, { status: 502, headers: NO_STORE });
  }
}
