import { auth } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

import { SITE_AUTH_BACKEND_ENABLED } from "@/lib/auth-config";
import { createPortalSession, dodoConfig, dodoCustomerFor } from "@/lib/billing/dodo";

export const dynamic = "force-dynamic";

/** Opens Dodo's billing portal (invoices, payment method, cancel) for the signed-in user. */
export async function GET(req: NextRequest) {
  const config = dodoConfig();
  if (!SITE_AUTH_BACKEND_ENABLED || !config) {
    return NextResponse.redirect(new URL("/account?billing=unavailable", req.nextUrl.origin));
  }
  const { userId } = await auth();
  if (!userId) {
    const signIn = new URL("/sign-in", req.nextUrl.origin);
    signIn.searchParams.set("redirect_url", req.url);
    return NextResponse.redirect(signIn);
  }
  const back = new URL("/account", req.nextUrl.origin).toString();
  try {
    const customer = await dodoCustomerFor(userId);
    if (!customer) return NextResponse.redirect(new URL("/account?billing=none", req.nextUrl.origin));
    return NextResponse.redirect(await createPortalSession(config, customer, back));
  } catch {
    return NextResponse.redirect(new URL("/account?billing=unavailable", req.nextUrl.origin));
  }
}
