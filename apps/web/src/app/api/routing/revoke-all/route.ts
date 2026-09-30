import { auth } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";

export const dynamic = "force-dynamic";

function response(error: string, status: number) {
  return NextResponse.json({ error }, { status, headers: { "Cache-Control": "no-store" } });
}

export async function DELETE(request: NextRequest) {
  // Incident cleanup remains available after new account connections stop.
  if (!SITE_AUTH_BACKEND_ENABLED) {
    return response("desktop_bridge_unavailable", 503);
  }
  if (request.headers.get("origin") !== new URL(request.url).origin) {
    return response("origin_rejected", 403);
  }
  const { userId, getToken } = await auth();
  if (!userId) return response("unauthorized", 401);
  const token = await getToken();
  if (!token) return response("unauthorized", 401);
  const base = process.env.LINK_ROUTING_BRIDGE_URL;
  if (!base) return response("desktop_bridge_unavailable", 503);
  let link: URL;
  try {
    link = new URL(base);
    if (link.protocol !== "https:" || link.username || link.password || link.pathname !== "/" || link.search || link.hash) {
      return response("desktop_bridge_unavailable", 503);
    }
    link = new URL("/v1/routing/desktop-sessions", link);
  } catch {
    return response("desktop_bridge_unavailable", 503);
  }
  try {
    const upstream = await fetch(link, {
      method: "DELETE",
      headers: { Authorization: `Bearer ${token}` },
      cache: "no-store",
      redirect: "manual",
      signal: AbortSignal.timeout(5_000),
    });
    if (upstream.status !== 204) return response("desktop_bridge_unavailable", 502);
    return new NextResponse(null, { status: 204, headers: { "Cache-Control": "no-store" } });
  } catch {
    return response("desktop_bridge_unavailable", 502);
  }
}
