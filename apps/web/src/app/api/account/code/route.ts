import { auth } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

import { ACCOUNT_CODE } from "@/lib/account-connect";
import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";
import { linkUrl, NO_STORE } from "@/lib/link";

export const dynamic = "force-dynamic";

const RANDOM_32_B64URL = /^[A-Za-z0-9_-]{43}$/;

function fail(error: string, status: number) {
  return NextResponse.json({ error }, { status, headers: NO_STORE });
}

/**
 * Mints a one-use sign-in code for the CLI or Desktop. The code is bound to
 * the client's PKCE challenge in Link, so only the client that started the
 * request can turn it into a credential.
 */
export async function POST(request: NextRequest) {
  if (!SITE_AUTH_BACKEND_ENABLED) return fail("account_unavailable", 503);
  if (request.headers.get("origin") !== new URL(request.url).origin) return fail("origin_rejected", 403);
  const text = await request.text();
  if (text.length > 256) return fail("invalid_request", 400);
  let body: unknown;
  try {
    body = JSON.parse(text);
  } catch {
    return fail("invalid_request", 400);
  }
  if (
    !body ||
    typeof body !== "object" ||
    Array.isArray(body) ||
    Object.keys(body).sort().join(",") !== "code_challenge,state"
  ) {
    return fail("invalid_request", 400);
  }
  const { state, code_challenge } = body as Record<string, unknown>;
  if (typeof state !== "string" || typeof code_challenge !== "string" || !RANDOM_32_B64URL.test(state) || !RANDOM_32_B64URL.test(code_challenge)) {
    return fail("invalid_request", 400);
  }

  const { userId, getToken } = await auth();
  if (!userId) return fail("unauthorized", 401);
  const token = await getToken();
  if (!token) return fail("unauthorized", 401);
  const target = linkUrl("/v1/account/authorize");
  if (!target) return fail("account_unavailable", 503);

  try {
    const upstream = await fetch(target, {
      method: "POST",
      headers: { Authorization: `Bearer ${token}`, "Content-Type": "application/json" },
      body: JSON.stringify({ state, code_challenge }),
      cache: "no-store",
      redirect: "manual",
      signal: AbortSignal.timeout(5_000),
    });
    if (upstream.status === 429) return fail("too_many_requests", 429);
    if (!upstream.ok) return fail("account_unavailable", 502);
    const payload = (await upstream.json()) as { code?: unknown };
    if (typeof payload.code !== "string" || !ACCOUNT_CODE.test(payload.code)) return fail("account_unavailable", 502);
    return NextResponse.json({ code: payload.code }, { headers: NO_STORE });
  } catch {
    return fail("account_unavailable", 502);
  }
}
