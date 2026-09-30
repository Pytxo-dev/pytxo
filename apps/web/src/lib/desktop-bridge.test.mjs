import assert from "node:assert/strict";
import { test } from "node:test";

import {
  desktopAccountUrl,
  desktopBridgeAllowed,
  desktopBridgeParams,
  desktopSignInUrl,
  desktopSignUpUrl,
} from "./desktop-bridge.ts";

const state = "A".repeat(43);
const challenge = "B".repeat(43);

test("Desktop PKCE state survives account, sign-in and sign-up redirects", () => {
  const source = new URLSearchParams({
    deck_callback: "pytxo-deck",
    state,
    code_challenge: challenge,
  });
  const bridge = desktopBridgeParams(source);
  assert.deepEqual(bridge, { state, codeChallenge: challenge });
  for (const url of [desktopAccountUrl(bridge), desktopSignInUrl(bridge), desktopSignUpUrl(bridge)]) {
    const params = new URL(url, "https://pytxo.com").searchParams;
    assert.deepEqual(desktopBridgeParams(params), bridge);
    assert.equal(params.has("token"), false);
  }
});

test("invalid or incomplete callbacks cannot become a Desktop return", () => {
  for (const params of [
    new URLSearchParams({ deck_callback: "pytxo-deck" }),
    new URLSearchParams({ deck_callback: "other", state, code_challenge: challenge }),
    new URLSearchParams({ deck_callback: "pytxo-deck", state: "short", code_challenge: challenge }),
    new URLSearchParams({ deck_callback: "pytxo-deck", state, code_challenge: "short" }),
  ]) {
    assert.equal(desktopBridgeParams(params), null);
  }
  assert.equal(desktopAccountUrl(null), "/account");
  assert.equal(desktopSignInUrl(null), "/sign-in");
  assert.equal(desktopSignUpUrl(null), "/sign-up");
});

test("Desktop recovery return survives sign-in and remains available only for an exact recovery request", () => {
  const source = new URLSearchParams({
    deck_callback: "pytxo-deck",
    state,
    code_challenge: challenge,
    deck_recovery: "1",
  });
  const bridge = desktopBridgeParams(source);
  assert.deepEqual(bridge, { state, codeChallenge: challenge, recoveryOnly: true });
  assert.equal(desktopBridgeAllowed(bridge, false), true);
  assert.equal(desktopBridgeAllowed(desktopBridgeParams(new URLSearchParams({ deck_callback: "pytxo-deck", state, code_challenge: challenge })), false), false);
  assert.equal(desktopBridgeAllowed(bridge, true), true);
  for (const url of [desktopAccountUrl(bridge), desktopSignInUrl(bridge), desktopSignUpUrl(bridge)]) {
    assert.deepEqual(desktopBridgeParams(new URL(url, "https://pytxo.com").searchParams), bridge);
  }
  source.set("deck_recovery", "yes");
  assert.equal(desktopBridgeParams(source), null);
});
