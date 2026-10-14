import { NextRequest, NextResponse } from "next/server";

export const runtime = "nodejs";

// sloplint-disable-next-line SEC-006 -- forwards the raw body and signature header; Link verifies them (services/pytxo-link/src/paddle.rs)
export async function POST(req: NextRequest) {
  const linkBase = process.env.LINK_ADMIN_URL?.trim();
  if (!linkBase) {
    return NextResponse.json({ error: "Webhook proxy not configured" }, { status: 503 });
  }

  let target: URL;
  try {
    target = new URL("/v1/webhooks/paddle", linkBase);
  } catch {
    return NextResponse.json({ error: "Webhook proxy not configured" }, { status: 503 });
  }

  const headers = new Headers();
  const signature = req.headers.get("paddle-signature");
  const contentType = req.headers.get("content-type");
  if (signature) headers.set("paddle-signature", signature);
  if (contentType) headers.set("content-type", contentType);

  try {
    const upstream = await fetch(target, {
      method: "POST",
      headers,
      body: await req.arrayBuffer(),
      cache: "no-store",
      redirect: "manual",
      signal: AbortSignal.timeout(15_000),
    });
    const responseHeaders = new Headers();
    const upstreamContentType = upstream.headers.get("content-type");
    if (upstreamContentType) responseHeaders.set("content-type", upstreamContentType);
    return new Response(await upstream.arrayBuffer(), {
      status: upstream.status,
      statusText: upstream.statusText,
      headers: responseHeaders,
    });
  } catch {
    return NextResponse.json({ error: "Webhook proxy unavailable" }, { status: 502 });
  }
}
