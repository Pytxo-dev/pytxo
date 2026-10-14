// Server-side only: route handlers import this. Never import from a client component.
/**
 * Dodo Payments is the merchant of record for Pytxo Pro. The website creates
 * checkout and billing-portal sessions; Link learns about subscriptions from
 * Dodo's signed webhooks (services/pytxo-link/src/dodo.rs), keyed by the Clerk
 * user id this module puts in checkout metadata.
 */
const BASES = {
  test: "https://test.dodopayments.com",
  live: "https://live.dodopayments.com",
} as const;

export type DodoConfig = { apiKey: string; productId: string; base: string };

/** Null when billing is not configured on this deployment. */
export function dodoConfig(): DodoConfig | null {
  const apiKey = process.env.DODO_API_KEY?.trim();
  const productId = process.env.DODO_PRODUCT_PRO?.trim();
  if (!apiKey || !productId) return null;
  const mode = process.env.DODO_ENVIRONMENT === "live" ? "live" : "test";
  return { apiKey, productId, base: BASES[mode] };
}

async function dodoPost(config: DodoConfig, path: string, body?: unknown): Promise<unknown> {
  const res = await fetch(new URL(path, config.base), {
    method: "POST",
    headers: {
      Authorization: `Bearer ${config.apiKey}`,
      ...(body === undefined ? {} : { "Content-Type": "application/json" }),
    },
    body: body === undefined ? undefined : JSON.stringify(body),
    cache: "no-store",
    signal: AbortSignal.timeout(10_000),
  });
  if (!res.ok) throw new Error(`Dodo ${path.split("?")[0]} answered ${res.status}`);
  return res.json();
}

function httpsUrl(value: unknown): string {
  if (typeof value !== "string") throw new Error("Dodo returned no URL");
  const url = new URL(value);
  if (url.protocol !== "https:") throw new Error("Dodo returned a non-HTTPS URL");
  return url.toString();
}

export async function createProCheckout(
  config: DodoConfig,
  input: { userId: string; email?: string; name?: string; returnUrl: string },
): Promise<string> {
  const session = (await dodoPost(config, "/checkouts", {
    product_cart: [{ product_id: config.productId, quantity: 1 }],
    ...(input.email ? { customer: { email: input.email, ...(input.name ? { name: input.name } : {}) } } : {}),
    return_url: input.returnUrl,
    metadata: { user_id: input.userId },
  })) as { checkout_url?: unknown };
  return httpsUrl(session.checkout_url);
}

export async function createPortalSession(
  config: DodoConfig,
  customerId: string,
  returnUrl: string,
): Promise<string> {
  const query = new URLSearchParams({ return_url: returnUrl });
  const session = (await dodoPost(
    config,
    `/customers/${encodeURIComponent(customerId)}/customer-portal/session?${query}`,
  )) as { link?: unknown };
  return httpsUrl(session.link);
}

/** The Dodo customer behind a user's subscription, from Link (admin key). */
export async function dodoCustomerFor(userId: string): Promise<string | null> {
  const base = process.env.LINK_ADMIN_URL?.trim();
  const key = process.env.LINK_ADMIN_KEY?.trim();
  if (!base || !key) return null;
  const res = await fetch(new URL(`/v1/admin/billing/${encodeURIComponent(userId)}/dodo-customer`, base), {
    headers: { "x-link-admin-key": key },
    cache: "no-store",
    signal: AbortSignal.timeout(5_000),
  });
  if (res.status === 404) return null;
  if (!res.ok) throw new Error(`Link answered ${res.status}`);
  const body = (await res.json()) as { customer_id?: unknown };
  return typeof body.customer_id === "string" ? body.customer_id : null;
}
