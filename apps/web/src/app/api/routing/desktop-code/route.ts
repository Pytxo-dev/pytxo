import { auth } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";
import { desktopBridgeAllowed, desktopBridgeParams } from "@/lib/desktop-bridge";

export const dynamic = "force-dynamic";

function response(error: string, status: number) {
  return NextResponse.json({ error }, { status, headers: { "Cache-Control": "no-store" } });
}

export async function POST(request: NextRequest) {
  if (!SITE_AUTH_BACKEND_ENABLED) {
    return response("desktop_bridge_unavailable", 503);
  }
  if (request.headers.get("origin") !== new URL(request.url).origin) {
    return response("origin_rejected", 403);
  }
  let body: Record<string, unknown>;
  try {
    const reader = request.body?.getReader();
    if (!reader) return response("invalid_request", 400);
    const chunks: Uint8Array[] = [];
    let count = 0;
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      count += value.byteLength;
      if (count > 512) {
        await reader.cancel();
        return response("invalid_request", 400);
      }
      chunks.push(value);
    }
    const bytes = new Uint8Array(count);
    let offset = 0;
    for (const chunk of chunks) {
      bytes.set(chunk, offset);
      offset += chunk.byteLength;
    }
    body = JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(bytes)) as Record<string, unknown>;
  } catch {
    return response("invalid_request", 400);
  }
  if (
    !body ||
    typeof body !== "object" ||
    Array.isArray(body) ||
    !["code_challenge,state", "code_challenge,recovery_only,state"].includes(Object.keys(body).sort().join(",")) ||
    typeof body.state !== "string" ||
    typeof body.code_challenge !== "string" ||
    ("recovery_only" in body && body.recovery_only !== true)
  ) {
    return response("invalid_request", 400);
  }
  const params = new URLSearchParams({
    deck_callback: "pytxo-deck",
    state: body.state,
    code_challenge: body.code_challenge,
  });
  if (body.recovery_only === true) params.set("deck_recovery", "1");
  if (!desktopBridgeAllowed(desktopBridgeParams(params), process.env.NEXT_PUBLIC_ROUTING_BRIDGE_EXPERIMENT === "1")) {
    return response("desktop_bridge_unavailable", 503);
  }

  const { userId, getToken } = await auth();
  if (!userId) return response("unauthorized", 401);
  // Use the ordinary Clerk session token. Custom JWT templates omit the
  // session-bound `sid` required by Link. Deployment must add the configured
  // routing audience to the session token itself.
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
    link = new URL("/v1/routing/desktop-authorizations", link);
  } catch {
    return response("desktop_bridge_unavailable", 503);
  }
  try {
    const upstream = await fetch(link, {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        state: body.state,
        code_challenge: body.code_challenge,
        ...(body.recovery_only === true ? { recovery_only: true } : {}),
      }),
      cache: "no-store",
      redirect: "manual",
      signal: AbortSignal.timeout(5_000),
    });
    if (!upstream.ok) return response("desktop_bridge_unavailable", 502);
    const payload = (await upstream.json()) as { code?: unknown; expires_at?: unknown };
    if (
      typeof payload.code !== "string" ||
      !/^pdc1_[A-Za-z0-9_-]{43}$/.test(payload.code) ||
      typeof payload.expires_at !== "string"
    ) {
      return response("desktop_bridge_unavailable", 502);
    }
    return NextResponse.json(
      { code: payload.code, expires_at: payload.expires_at },
      { headers: { "Cache-Control": "no-store" } },
    );
  } catch {
    return response("desktop_bridge_unavailable", 502);
  }
}
