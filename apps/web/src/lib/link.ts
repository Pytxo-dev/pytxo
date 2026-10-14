// Server-side only: route handlers and server components import this.
/**
 * Pytxo Link base URL for account perks. HTTPS in production; plain HTTP is
 * accepted only for a loopback Link during local development.
 */
export function linkUrl(path: string): URL | null {
  const raw = process.env.LINK_URL ?? process.env.NEXT_PUBLIC_LINK_URL ?? "https://link.pytxo.com";
  try {
    const base = new URL(raw);
    const loopback = base.hostname === "127.0.0.1" || base.hostname === "localhost";
    if (base.username || base.password || base.search || base.hash) return null;
    if (base.protocol !== "https:" && !(loopback && base.protocol === "http:")) return null;
    return new URL(path, base);
  } catch {
    return null;
  }
}

export const NO_STORE = { "Cache-Control": "no-store" } as const;
