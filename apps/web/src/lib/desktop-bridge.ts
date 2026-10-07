export type DesktopBridgeParams = {
  state: string;
  codeChallenge: string;
  recoveryOnly?: true;
};

const RANDOM_32_B64URL = /^[A-Za-z0-9_-]{43}$/;

/** Only a Desktop-started PKCE request is carried across Clerk redirects. */
export function desktopBridgeParams(
  params: Pick<URLSearchParams, "get">,
): DesktopBridgeParams | null {
  if (params.get("deck_callback") !== "pytxo-deck") return null;
  const state = params.get("state");
  const codeChallenge = params.get("code_challenge");
  const recovery = params.get("deck_recovery");
  if (!state || !codeChallenge) return null;
  if (!RANDOM_32_B64URL.test(state) || !RANDOM_32_B64URL.test(codeChallenge)) return null;
  if (recovery !== null && recovery !== "1") return null;
  return recovery === "1" ? { state, codeChallenge, recoveryOnly: true } : { state, codeChallenge };
}

export function desktopBridgeAllowed(bridge: DesktopBridgeParams | null, bridgeEnabled: boolean): boolean {
  return !!bridge && (bridgeEnabled || bridge.recoveryOnly === true);
}

export function desktopAccountUrl(bridge: DesktopBridgeParams | null): string {
  if (!bridge) return "/account";
  const params = new URLSearchParams({
    deck_callback: "pytxo-deck",
    state: bridge.state,
    code_challenge: bridge.codeChallenge,
  });
  if (bridge.recoveryOnly) params.set("deck_recovery", "1");
  return `/account?${params}`;
}

export function desktopSignInUrl(bridge: DesktopBridgeParams | null): string {
  return desktopAccountUrl(bridge).replace(/^\/account/, "/sign-in");
}

export function desktopSignUpUrl(bridge: DesktopBridgeParams | null): string {
  return desktopAccountUrl(bridge).replace(/^\/account/, "/sign-up");
}
