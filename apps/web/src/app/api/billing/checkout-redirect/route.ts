import { auth, currentUser } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";
import { createProCheckout, dodoConfig } from "@/lib/billing/dodo";
import { isProTier } from "@/lib/billing/pro";
import { linkUrl, NO_STORE } from "@/lib/link";

export const dynamic = "force-dynamic";

/** The signed-in user's Link tier, or null when Link cannot say. */
async function currentTier(token: string): Promise<string | null> {
  const target = linkUrl("/v1/entitlements/status");
  if (!target) return null;
  try {
    const res = await fetch(target, {
      headers: { Authorization: `Bearer ${token}` },
      cache: "no-store",
      signal: AbortSignal.timeout(5_000),
    });
    if (!res.ok) return null;
    const body = (await res.json()) as { tier?: unknown };
    return typeof body.tier === "string" ? body.tier : null;
  } catch {
    return null;
  }
}

/** Starts a Dodo checkout for Pytxo Pro (`?plan=pro`, the only paid plan). */
export async function GET(req: NextRequest) {
  const config = dodoConfig();
  if (!SITE_AUTH_BACKEND_ENABLED || !config) {
    return NextResponse.redirect(new URL("/account?billing=unavailable", req.nextUrl.origin));
  }
  if (req.nextUrl.searchParams.get("plan") !== "pro") {
    return NextResponse.json({ error: "Invalid plan" }, { status: 400, headers: NO_STORE });
  }

  const { userId, getToken } = await auth();
  if (!userId) {
    const signIn = new URL("/sign-in", req.nextUrl.origin);
    signIn.searchParams.set("redirect_url", req.url);
    return NextResponse.redirect(signIn);
  }
  const token = await getToken();
  if (token && isProTier(await currentTier(token))) {
    return NextResponse.redirect(new URL("/account?pro=active", req.nextUrl.origin));
  }

  const user = await currentUser();
  try {
    const checkoutUrl = await createProCheckout(config, {
      userId,
      email: user?.primaryEmailAddress?.emailAddress ?? user?.emailAddresses[0]?.emailAddress,
      name: user?.fullName ?? undefined,
      returnUrl: new URL("/account?upgraded=1", req.nextUrl.origin).toString(),
    });
    return NextResponse.redirect(checkoutUrl);
  } catch {
    return NextResponse.redirect(new URL("/account?billing=unavailable", req.nextUrl.origin));
  }
}
