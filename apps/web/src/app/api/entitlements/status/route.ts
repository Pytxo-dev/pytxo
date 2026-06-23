import { auth } from "@clerk/nextjs/server";
import { NextResponse } from "next/server";

export async function GET() {
  const { userId, getToken } = await auth();
  if (!userId) {
    return NextResponse.json({ error: "unauthorized" }, { status: 401 });
  }

  const linkBase = process.env.LINK_ADMIN_URL ?? process.env.NEXT_PUBLIC_LINK_URL ?? "https://link.pytxo.com";
  const token = await getToken();
  if (!token) {
    return NextResponse.json({ error: "no_session_token" }, { status: 401 });
  }

  try {
    const res = await fetch(`${linkBase.replace(/\/$/, "")}/v1/entitlements/status`, {
      headers: {
        Authorization: `Bearer ${token}`,
        "x-pytxo-user-id": userId,
      },
      next: { revalidate: 60 },
    });
    if (!res.ok) {
      return NextResponse.json(
        { error: "link_unreachable", status: res.status },
        { status: 502 },
      );
    }
    const data = await res.json();
    return NextResponse.json({ userId, ...data });
  } catch (e) {
    return NextResponse.json(
      { error: "link_fetch_failed", detail: String(e) },
      { status: 502 },
    );
  }
}
