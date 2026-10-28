import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  commentBody,
  glob,
  isAgentPr,
  LICENSE_PUBLIC_KEY,
  licenseGate,
  LICENSING,
  localName,
  MARKER,
  parsePatterns,
  selectPrs,
  upsertComment,
  verifyLicense,
} from "./pytxo-action.mjs";

const script = fileURLToPath(new URL("./pytxo-action.mjs", import.meta.url));
const REPO = "acme/shop";

function pr(number, ref, { login = "matt", base = "main", fork = false } = {}) {
  return {
    number,
    user: { login },
    head: { ref, sha: `${number}`.padStart(40, "0"), repo: { full_name: fork ? "someone/shop" : REPO } },
    base: { ref: base, repo: { full_name: REPO } },
  };
}

test("patterns match like the CLI's: * crosses slashes, a trailing / means everything under it", () => {
  assert.ok(glob("claude/*", "claude/fix/tax"));
  assert.ok(glob("agent/?", "agent/a"));
  assert.ok(!glob("codex/*", "codexx/a"));
  assert.ok(!glob("claude/*", "my-claude/a"));
  assert.deepEqual(parsePatterns("claude/, bots/*"), ["claude/*", "bots/*"]);
  assert.equal(parsePatterns("").length, 7);
});

test("agent pull requests are found by branch prefix or by the agent's account", () => {
  assert.ok(isAgentPr(pr(1, "codex/discount")));
  assert.ok(isAgentPr(pr(2, "copilot/fix-tax")));
  assert.ok(isAgentPr(pr(3, "add-tax-rounding", { login: "Copilot" })));
  assert.ok(isAgentPr(pr(4, "feature", { login: "google-labs-jules[bot]" })));
  assert.ok(!isAgentPr(pr(5, "matt/readme")));
  assert.ok(isAgentPr(pr(5, "matt/readme"), ["matt/*"]));
});

test("the other agent PRs into the same base join, oldest first, this one last", () => {
  const current = pr(14, "codex/discount");
  const open = [
    current,
    pr(15, "claude/rename"),
    pr(9, "cursor/docs"),
    pr(11, "matt/readme"),
    pr(12, "claude/tax", { base: "release" }),
    pr(13, "copilot/fork-fix", { fork: true }),
  ];
  const { prs, notes, reason } = selectPrs(current, open);
  assert.equal(reason, undefined);
  assert.deepEqual(
    prs.map((p) => p.number),
    [9, 15, 14],
  );
  assert.deepEqual(notes, ["1 agent pull request from forks left out."]);
});

test("a fork PR is checked with the base repo's agent PRs, never the other way round", () => {
  const fromFork = pr(20, "codex/x", { fork: true });
  const { prs } = selectPrs(fromFork, [pr(18, "claude/y"), pr(19, "codex/z", { fork: true }), fromFork]);
  assert.deepEqual(
    prs.map((p) => p.number),
    [18, 20],
  );
});

test("nothing to check: not an agent PR, or the only one", () => {
  assert.match(selectPrs(pr(3, "matt/readme"), []).reason, /#3 is not an agent pull request/);
  const alone = selectPrs(pr(4, "claude/a"), [pr(4, "claude/a"), pr(5, "matt/b")]);
  assert.match(alone.reason, /only open agent pull request into main/);
});

test("at most ten PRs at once, keeping the newest", () => {
  const open = Array.from({ length: 14 }, (_, i) => pr(i + 1, `claude/b${i + 1}`));
  const { prs, notes } = selectPrs(pr(30, "codex/new"), open);
  assert.equal(prs.length, 10);
  assert.equal(prs[0].number, 6);
  assert.equal(prs.at(-1).number, 30);
  assert.deepEqual(notes, ["5 older agent pull requests left out (at most 10 at once)."]);
});

test("local branch names keep the PR's own name when it is safe and unique", () => {
  const taken = new Set(["main"]);
  assert.equal(localName(pr(1, "claude/fix-tax"), taken, "main"), "claude/fix-tax");
  assert.equal(localName(pr(2, "claude/fix-tax"), taken, "main"), "pr-2");
  assert.equal(localName(pr(3, "main"), taken, "main"), "pr-3");
  assert.equal(localName(pr(4, "-x"), taken, "main"), "pr-4");
  assert.equal(localName(pr(5, "a..b"), taken, "main"), "pr-5");
});

function fakeGitHub(comments, { status = 200 } = {}) {
  const calls = [];
  const fetchImpl = async (url, init = {}) => {
    calls.push({ url, method: init.method || "GET", body: init.body && JSON.parse(init.body) });
    if (status !== 200) return { ok: false, status, json: async () => ({}) };
    return { ok: true, status: 200, json: async () => comments };
  };
  return { calls, fetchImpl };
}

test("one comment: created the first time, updated in place after", async () => {
  const target = { apiUrl: "https://api", repo: REPO, number: 14, token: "t", body: `${MARKER}\nverdict` };
  const fresh = fakeGitHub([{ id: 1, body: "someone else" }]);
  assert.equal(await upsertComment(fresh.fetchImpl, target), "created");
  assert.equal(fresh.calls[1].method, "POST");
  assert.equal(fresh.calls[1].url, "https://api/repos/acme/shop/issues/14/comments");

  const again = fakeGitHub([{ id: 1, body: "someone else" }, { id: 7, body: `${MARKER}\nold` }]);
  assert.equal(await upsertComment(again.fetchImpl, target), "updated");
  assert.equal(again.calls[1].method, "PATCH");
  assert.equal(again.calls[1].url, "https://api/repos/acme/shop/issues/comments/7");
  assert.equal(again.calls[1].body.body, target.body);

  const none = fakeGitHub([]);
  assert.equal(await upsertComment(none.fetchImpl, { ...target, create: false }), "none");
  assert.equal(none.calls.length, 1);

  const readOnly = fakeGitHub([], { status: 403 });
  assert.equal(await upsertComment(readOnly.fetchImpl, target), "forbidden");
});

test("the comment names the PRs, then carries the CLI's verdict or the error", () => {
  const prs = [
    { number: 9, branch: "claude/rename" },
    { number: 14, branch: "codex/discount" },
  ];
  const ok = commentBody({ prs, markdown: "### ⛔ BLOCKED: x\n", stderr: "", code: "2" });
  assert.ok(ok.startsWith(`${MARKER}\nChecked together: #9 \`claude/rename\` · #14 \`codex/discount\`\n`));
  assert.ok(ok.includes("### ⛔ BLOCKED: x"));
  const broken = commentBody({ prs, markdown: "", stderr: "npm error 404 pytxo@9\n", code: "1" });
  assert.ok(broken.includes("could not run the check (exit 1)"));
  assert.ok(broken.includes("npm error 404 pytxo@9"));
  assert.ok(commentBody({ reason: "nothing to combine" }).includes("Nothing to check"));
});

// discover, end to end: a real origin with refs/pull/N/head, a local API.
// Async: a synchronous child would block the API server in this process.
function node(args, options) {
  return new Promise((resolve) => {
    const child = spawn(process.execPath, args, options);
    let out = "";
    child.stdout.on("data", (d) => (out += d));
    child.stderr.on("data", (d) => (out += d));
    child.on("close", (status) => resolve({ status, out }));
  });
}

function sh(cwd, ...args) {
  const r = spawnSync("git", args, { cwd, encoding: "utf8" });
  assert.equal(r.status, 0, `git ${args.join(" ")}: ${r.stderr}`);
  return r.stdout.trim();
}

test("discover fetches the agent PRs as local branches and the base", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "pytxo-action-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const work = path.join(root, "work");
  fs.mkdirSync(work);
  sh(work, "init", "-q", "-b", "main");
  sh(work, "config", "user.email", "t@t");
  sh(work, "config", "user.name", "t");
  fs.writeFileSync(path.join(work, "a.txt"), "a\n");
  sh(work, "add", "-A");
  sh(work, "commit", "-q", "-m", "base");
  const heads = {};
  for (const [n, name] of [
    [9, "claude/rename"],
    [14, "codex/discount"],
  ]) {
    sh(work, "checkout", "-q", "-b", name, "main");
    fs.writeFileSync(path.join(work, `${n}.txt`), `${n}\n`);
    sh(work, "add", "-A");
    sh(work, "commit", "-q", "-m", name);
    heads[n] = sh(work, "rev-parse", "HEAD");
    sh(work, "checkout", "-q", "main");
  }
  const origin = path.join(root, "origin.git");
  sh(root, "clone", "-q", "--bare", work, origin);
  sh(origin, "update-ref", "refs/pull/9/head", heads[9]);
  sh(origin, "update-ref", "refs/pull/14/head", heads[14]);
  sh(origin, "branch", "-D", "claude/rename", "codex/discount");
  const clone = path.join(root, "clone");
  sh(root, "clone", "-q", "--depth", "1", `file://${origin.replace(/\\/g, "/")}`, clone);
  sh(clone, "checkout", "-q", "--detach");

  const pulls = [pr(9, "claude/rename"), pr(14, "codex/discount"), pr(11, "matt/readme")];
  const server = http.createServer((req, res) => {
    assert.equal(req.headers.authorization, "Bearer secret");
    res.setHeader("content-type", "application/json");
    res.end(JSON.stringify(req.url.startsWith(`/repos/${REPO}/pulls`) ? pulls : []));
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => server.closeAllConnections() || server.close());

  const event = path.join(root, "event.json");
  fs.writeFileSync(event, JSON.stringify({ pull_request: pulls[1] }));
  const outputs = path.join(root, "outputs");
  fs.writeFileSync(outputs, "");
  const run = await node([script, "discover"], {
    cwd: clone,
    env: {
      ...process.env,
      GITHUB_EVENT_NAME: "pull_request",
      GITHUB_EVENT_PATH: event,
      GITHUB_OUTPUT: outputs,
      GITHUB_REPOSITORY: REPO,
      GITHUB_API_URL: `http://127.0.0.1:${server.address().port}`,
      RUNNER_TEMP: root,
      PYTXO_TOKEN: "secret",
    },
  });
  assert.equal(run.status, 0, run.out);
  assert.match(fs.readFileSync(outputs, "utf8"), /skip=false\nbase=main\n/);
  assert.equal(fs.readFileSync(path.join(root, "pytxo-branches.txt"), "utf8"), "claude/rename\ncodex/discount\n");
  assert.equal(sh(clone, "rev-parse", "refs/heads/claude/rename"), heads[9]);
  assert.equal(sh(clone, "rev-parse", "refs/heads/codex/discount"), heads[14]);
  assert.equal(sh(clone, "rev-parse", "--is-shallow-repository"), "false");
  sh(clone, "merge-base", "main", "codex/discount");

  const refused = spawnSync(process.execPath, [script, "discover"], {
    cwd: clone,
    encoding: "utf8",
    env: { ...process.env, GITHUB_EVENT_NAME: "pull_request_target", GITHUB_EVENT_PATH: event, GITHUB_OUTPUT: outputs },
  });
  assert.equal(refused.status, 1);
  assert.match(refused.stdout, /refusing to run on pull_request_target/);
});

test("Pro licensing is built but switched off", () => {
  assert.equal(LICENSING, false);
  assert.equal(LICENSE_PUBLIC_KEY, null);
});

test("license keys verify offline; private repositories get a 14-day trial", () => {
  const { publicKey, privateKey } = crypto.generateKeyPairSync("ed25519");
  const sign = (claims) => {
    const payload = Buffer.from(JSON.stringify(claims)).toString("base64url");
    return `pytxo1.${payload}.${crypto.sign(null, Buffer.from(payload), privateKey).toString("base64url")}`;
  };
  const now = new Date("2026-11-20T00:00:00Z");
  const key = sign({ repo: "acme/shop", exp: "2027-01-01" });
  assert.deepEqual(verifyLicense(key, "acme/shop", now, publicKey), { valid: true });
  assert.ok(verifyLicense(sign({ repo: "acme/*", exp: null }), "acme/api", now, publicKey).valid);
  assert.match(verifyLicense(key, "other/shop", now, publicKey).reason, /for acme\/shop/);
  assert.match(verifyLicense(sign({ repo: "acme/shop", exp: "2026-11-01" }), "acme/shop", now, publicKey).reason, /expired/);
  const forged = `${key.slice(0, key.lastIndexOf(".") + 1)}${Buffer.alloc(64).toString("base64url")}`;
  assert.match(verifyLicense(forged, "acme/shop", now, publicKey).reason, /signature/);
  assert.equal(verifyLicense(key, "acme/shop", now, null).valid, false);

  const gate = (opts) => licenseGate({ repo: "acme/shop", now, publicKey, ...opts });
  assert.deepEqual(gate({ isPrivate: false }), { allowed: true });
  const trial = gate({ isPrivate: true, trialStart: new Date("2026-11-10T00:00:00Z") });
  assert.equal(trial.allowed, true);
  assert.match(trial.notice, /4 days left/);
  const over = gate({ isPrivate: true, trialStart: new Date("2026-10-01T00:00:00Z") });
  assert.equal(over.allowed, false);
  assert.match(over.notice, /PYTXO_LICENSE/);
  assert.deepEqual(gate({ isPrivate: true, trialStart: new Date("2026-10-01T00:00:00Z"), key }), { allowed: true });
});
