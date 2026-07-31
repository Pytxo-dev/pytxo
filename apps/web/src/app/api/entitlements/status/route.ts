import { auth } from "@clerk/nextjs/server";
import { NextResponse } from "next/server";

type LinkHeaders = {
  Authorization: string;
  "x-pytxo-user-id": string;
};

async function linkFetch(path: string, headers: LinkHeaders) {
  const linkBase =
    process.env.LINK_ADMIN_URL ?? process.env.NEXT_PUBLIC_LINK_URL ?? "https://link.pytxo.com";
  return fetch(`${linkBase.replace(/\/$/, "")}${path}`, {
    headers,
    next: { revalidate: 60 },
  });
}

export async function GET() {
  if (
    !process.env.NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY ||
    !process.env.CLERK_SECRET_KEY
  ) {
    return NextResponse.json(
      { error: "account_auth_unavailable" },
      { status: 503 },
    );
  }

  const { userId, getToken } = await auth();
  if (!userId) {
    return NextResponse.json({ error: "unauthorized" }, { status: 401 });
  }

  const token = await getToken();
  if (!token) {
    return NextResponse.json({ error: "no_session_token" }, { status: 401 });
  }

  const linkHeaders: LinkHeaders = {
    Authorization: `Bearer ${token}`,
    "x-pytxo-user-id": userId,
  };

  try {
    const res = await linkFetch("/v1/entitlements/status", linkHeaders);
    if (!res.ok) {
      return NextResponse.json(
        { error: "link_unreachable", status: res.status },
        { status: 502 },
      );
    }
    const data = (await res.json()) as Record<string, unknown>;
    const orgId = typeof data.org_id === "string" ? data.org_id : null;

    if (orgId) {
      const [policyRes, seatsRes] = await Promise.all([
        linkFetch(`/v1/orgs/${encodeURIComponent(orgId)}/policy`, linkHeaders),
        linkFetch(`/v1/orgs/${encodeURIComponent(orgId)}/seats`, linkHeaders),
      ]);
      if (policyRes.ok) {
        data.org_policy = await policyRes.json();
      }
      if (seatsRes.ok) {
        data.org_seats = await seatsRes.json();
      }
    }

    return NextResponse.json({ userId, ...data });
  } catch (e) {
    return NextResponse.json(
      { error: "link_fetch_failed", detail: String(e) },
      { status: 502 },
    );
  }
}
