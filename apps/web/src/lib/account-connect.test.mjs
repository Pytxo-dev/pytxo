import assert from "node:assert/strict";
import { test } from "node:test";

import { connectParams, connectPath, returnUrl, safeConnectReturn } from "./account-connect.ts";

const R = "A".repeat(43);
const CODE = `pac1_${"b".repeat(43)}`;

test("cli requests need a high loopback port", () => {
  assert.deepEqual(connectParams(new URLSearchParams(`client=cli&state=${R}&code_challenge=${R}&port=54737`)), {
    client: "cli",
    state: R,
    codeChallenge: R,
    port: 54737,
  });
  assert.equal(connectParams(new URLSearchParams(`client=cli&state=${R}&code_challenge=${R}&port=80`)), null);
  assert.equal(connectParams(new URLSearchParams(`client=cli&state=${R}&code_challenge=${R}`)), null);
  assert.equal(connectParams(new URLSearchParams(`client=web&state=${R}&code_challenge=${R}`)), null);
  assert.equal(connectParams(new URLSearchParams(`client=desktop&state=short&code_challenge=${R}`)), null);
});

test("codes only go to localhost or the Desktop app link", () => {
  const cli = connectParams(new URLSearchParams(`client=cli&state=${R}&code_challenge=${R}&port=50000`));
  assert.equal(returnUrl(cli, CODE), `http://127.0.0.1:50000/callback?code=${CODE}&state=${R}`);
  const desktop = connectParams(new URLSearchParams(`client=desktop&state=${R}&code_challenge=${R}`));
  assert.equal(returnUrl(desktop, CODE), `pytxo-deck://account?code=${CODE}&state=${R}`);
  assert.throws(() => returnUrl(desktop, "javascript:alert(1)"));
});

test("sign-in returns only to a valid connect request", () => {
  const p = connectParams(new URLSearchParams(`client=desktop&state=${R}&code_challenge=${R}`));
  assert.equal(safeConnectReturn(connectPath(p)), connectPath(p));
  assert.equal(safeConnectReturn("https://evil.example/account/connect?x"), null);
  assert.equal(safeConnectReturn("/account/connect?client=cli&state=x"), null);
  assert.equal(safeConnectReturn(null), null);
});
