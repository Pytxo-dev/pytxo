/**
 * Browser half of `pytxo login` and Desktop sign-in. The client starts the
 * request with a PKCE challenge; this page only ever hands back a one-use
 * code bound to that challenge, to localhost (CLI) or the Desktop app link.
 */
export type ConnectClient = "cli" | "desktop";

export type ConnectParams = {
  client: ConnectClient;
  state: string;
  codeChallenge: string;
  /** Loopback port for the CLI callback. */
  port?: number;
};

const RANDOM_32_B64URL = /^[A-Za-z0-9_-]{43}$/;
export const ACCOUNT_CODE = /^pac1_[A-Za-z0-9_-]{43}$/;

export function connectParams(params: Pick<URLSearchParams, "get">): ConnectParams | null {
  const client = params.get("client");
  const state = params.get("state");
  const codeChallenge = params.get("code_challenge");
  if (client !== "cli" && client !== "desktop") return null;
  if (!state || !codeChallenge || !RANDOM_32_B64URL.test(state) || !RANDOM_32_B64URL.test(codeChallenge)) {
    return null;
  }
  if (client === "desktop") return { client, state, codeChallenge };
  const raw = params.get("port") ?? "";
  if (!/^\d{4,5}$/.test(raw)) return null;
  const port = Number(raw);
  if (port < 1024 || port > 65535) return null;
  return { client, state, codeChallenge, port };
}

export function connectPath(p: ConnectParams): string {
  const q = new URLSearchParams({ client: p.client, state: p.state, code_challenge: p.codeChallenge });
  if (p.port) q.set("port", String(p.port));
  return `/account/connect?${q}`;
}

/** Where the browser goes with the code. Never anywhere but localhost or the app link. */
export function returnUrl(p: ConnectParams, code: string): string {
  if (!ACCOUNT_CODE.test(code)) throw new Error("invalid code");
  const q = new URLSearchParams({ code, state: p.state });
  return p.client === "cli" ? `http://127.0.0.1:${p.port}/callback?${q}` : `pytxo-deck://account?${q}`;
}

/** Sign-in may only send people back to a valid connect request. */
export function safeConnectReturn(value: string | null): string | null {
  if (!value?.startsWith("/account/connect?")) return null;
  const parsed = connectParams(new URLSearchParams(value.slice("/account/connect?".length)));
  return parsed ? connectPath(parsed) : null;
}
