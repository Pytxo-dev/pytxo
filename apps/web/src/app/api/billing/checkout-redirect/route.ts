import { auth, currentUser } from "@clerk/nextjs/server";
import { NextRequest, NextResponse } from "next/server";

import { createCheckoutSession, type CheckoutPlan } from "@/lib/billing/mbcz-checkout";

const VALID_PLANS: CheckoutPlan[] = ["pro", "max", "ultra"];

export async function GET(req: NextRequest) {
  if (
    !process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY ||
    !process.env.CLERK_SECRET_KEY
  ) {
    return NextResponse.json(
      { error: "account_auth_unavailable" },
      { status: 503 },
    );
  }

  const { userId } = await auth();
  if (!userId) {
    const signIn = new URL("/account", req.nextUrl.origin);
    signIn.searchParams.set("redirect_url", req.url);
    return NextResponse.redirect(signIn);
  }

  const plan = req.nextUrl.searchParams.get("plan") as CheckoutPlan | null;
  if (!plan || !VALID_PLANS.includes(plan)) {
    return NextResponse.json({ error: "Invalid plan" }, { status: 400 });
  }

  const user = await currentUser();
  const email = user?.emailAddresses[0]?.emailAddress;

  const session = await createCheckoutSession({
    product: "pytxo",
    plan,
    customData: {
      user_id: userId,
      tier: plan,
    },
    customerEmail: email,
  });

  return NextResponse.redirect(session.checkoutUrl);
}
