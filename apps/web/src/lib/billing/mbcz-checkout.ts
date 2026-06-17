export type CheckoutPlan = "pro" | "max" | "ultra";

export async function createCheckoutSession(input: {
  product: string;
  plan: CheckoutPlan;
  customData: Record<string, string>;
  customerEmail?: string;
}): Promise<{ sessionId: string; checkoutUrl: string; expiresAt?: string }> {
  const base = process.env.MBCZ_CHECKOUT_BASE_URL ?? "https://mbcz.xyz";
  const apiKey = process.env.MBCZ_CHECKOUT_API_KEY;
  if (!apiKey) {
    throw new Error("MBCZ_CHECKOUT_API_KEY is not configured");
  }

  const res = await fetch(`${base}/api/checkout/sessions`, {
    method: "POST",
    headers: {
      Authorization: `Bearer ${apiKey}`,
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      product: input.product,
      plan: input.plan === "pro" ? "monthly" : "monthly",
      customData: input.customData,
      customerEmail: input.customerEmail,
    }),
  });

  if (!res.ok) {
    throw new Error(await res.text());
  }

  return res.json();
}
