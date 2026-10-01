import { NextRequest, NextResponse } from "next/server";

export const runtime = "nodejs";

const FORWARDED_HEADERS = [
  "content-type",
  "webhook-id",
  "webhook-timestamp",
  "webhook-signature",
] as const;

export async function POST(req: NextRequest) {
  const linkBase = process.env.LINK_ADMIN_URL?.trim();
  if (!linkBase) {
    return NextResponse.json(
      { error: "Webhook proxy not configured" },
      { status: 503 },
    );
  }

  let target: URL;
  try {
    target = new URL("/v1/webhooks/dodo", linkBase);
  } catch {
    return NextResponse.json(
      { error: "Webhook proxy not configured" },
      { status: 503 },
    );
  }

  const headers = new Headers();
  for (const name of FORWARDED_HEADERS) {
    const value = req.headers.get(name);
    if (value) headers.set(name, value);
  }

  try {
    const upstream = await fetch(target, {
      method: "POST",
      headers,
      body: await req.arrayBuffer(),
      cache: "no-store",
      redirect: "manual",
    });
    const responseHeaders = new Headers();
    const upstreamContentType = upstream.headers.get("content-type");
    if (upstreamContentType) {
      responseHeaders.set("content-type", upstreamContentType);
    }
    return new Response(await upstream.arrayBuffer(), {
      status: upstream.status,
      statusText: upstream.statusText,
      headers: responseHeaders,
    });
  } catch {
    return NextResponse.json(
      { error: "Webhook proxy unavailable" },
      { status: 502 },
    );
  }
}
