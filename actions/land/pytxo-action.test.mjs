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
  INSTALL_HINT,
  isAgentPr,
  jobOutcome,
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

/**
 * A GitHub API serving `pages` of comments, by the `page` query parameter.
 * /user answers only for a personal access token's `login`, as for GitHub's
 * own token it does not. `calls` leaves that lookup out.
 */
function fakeGitHub(pages, { status = 200, login } = {}) {
  const calls = [];
  const fetchImpl = async (url, init = {}) => {
    if (url.endsWith("/user")) return { ok: Boolean(login), status: login ? 200 : 403, json: async () => ({ login }) };
    calls.push({ url, method: init.method || "GET", body: init.body && JSON.parse(init.body) });
    if (status !== 200) return { ok: false, status, json: async () => ({}) };
    const page = Number(new URL(url).searchParams.get("page")) || 1;
    return { ok: true, status: 200, json: async () => pages[page - 1] ?? [] };
  };
  return { calls, fetchImpl };
}

const byBot = (id, body, login = "github-actions[bot]") => ({ id, body, user: { login, type: "Bot" } });
const byPerson = (id, body, login = "matt") => ({ id, body, user: { login, type: "User" } });

test("one comment: created the first time, updated in place after", async () => {
  const target = { apiUrl: "https://api", repo: REPO, number: 14, token: "t", body: `${MARKER}\nverdict` };
  // A person, or another bot, quoting the marker does not make the comment Pytxo's.
  const fresh = fakeGitHub([[byPerson(1, `${MARKER} pasted`), byBot(2, "another bot"), byBot(3, `> ${MARKER}`, "claude[bot]")]]);
  assert.equal(await upsertComment(fresh.fetchImpl, target), "created");
  assert.equal(fresh.calls.length, 2);
  assert.equal(fresh.calls[1].method, "POST");
  assert.equal(fresh.calls[1].url, "https://api/repos/acme/shop/issues/14/comments");

  // Pytxo's comment on the second page is found and updated in place.
  const chatter = Array.from({ length: 100 }, (_, i) => byPerson(i + 1, i === 0 ? MARKER : "lgtm"));
  const again = fakeGitHub([chatter, [byPerson(101, "lgtm"), byBot(107, `${MARKER}\nold`), byBot(108, `${MARKER}\nnewer`)]]);
  assert.equal(await upsertComment(again.fetchImpl, target), "updated");
  assert.deepEqual(
    again.calls.map((c) => c.method),
    ["GET", "GET", "PATCH"],
  );
  assert.match(again.calls[0].url, /\?per_page=100&page=1$/);
  assert.match(again.calls[1].url, /\?per_page=100&page=2$/);
  assert.equal(again.calls[2].url, "https://api/repos/acme/shop/issues/comments/107");
  assert.equal(again.calls[2].body.body, target.body);

  // A personal access token's comments are its user's, not a bot's.
  const pat = fakeGitHub([[byBot(4, `${MARKER}\nold`), byPerson(5, `${MARKER}\nold`, "matt-ci")]], { login: "matt-ci" });
  assert.equal(await upsertComment(pat.fetchImpl, target), "updated");
  assert.equal(pat.calls[1].url, "https://api/repos/acme/shop/issues/comments/5");

  // A full page with no match reads on; a short one ends the search.
  const twoPages = fakeGitHub([Array.from({ length: 100 }, (_, i) => byPerson(i + 1, "lgtm")), []]);
  assert.equal(await upsertComment(twoPages.fetchImpl, { ...target, create: false }), "none");
  assert.equal(twoPages.calls.length, 2);

  const readOnly = fakeGitHub([], { status: 403 });
  assert.equal(await upsertComment(readOnly.fetchImpl, target), "forbidden");
});

test("every comment names what it checked, then its notes, then the install line", () => {
  const prs = [
    { number: 9, branch: "claude/rename" },
    { number: 14, branch: "codex/discount" },
  ];
  const cli = `### ⛔ BLOCKED: x\n\n<details>\n\n</details>\n\n<sub>the CLI's install line</sub>\n<!-- pytxo-verdict {"label":"BLOCKED","implicated":[0]} -->\n`;
  const ok = commentBody({ prs, markdown: cli, stderr: "", code: "2", notes: ["1 agent pull request from forks left out."] });
  assert.equal(
    ok,
    [
      MARKER,
      "Checked together: #9 `claude/rename` · #14 `codex/discount`",
      "",
      "### ⛔ BLOCKED: x\n\n<details>\n\n</details>",
      "",
      "> 1 agent pull request from forks left out.",
      "",
      INSTALL_HINT,
      "",
    ].join("\n"),
  );
  const broken = commentBody({ prs, markdown: "", stderr: "npm error 404 pytxo@9\n", code: "1" });
  assert.ok(broken.includes("could not run the check (exit 1)"));
  assert.ok(broken.includes("npm error 404 pytxo@9"));
  const nothing = commentBody({ reason: "nothing to combine", notes: ["2 older agent pull requests left out."] });
  assert.ok(nothing.includes("Nothing to check"));
  assert.ok(nothing.endsWith(`\n\n> 2 older agent pull requests left out.\n\n${INSTALL_HINT}\n`));
  for (const body of [ok, broken, nothing]) {
    assert.ok(body.endsWith(`\n\n${INSTALL_HINT}\n`));
    assert.equal(body.split("<sub>").length, 2);
  }
});

/** The CLI's markdown: the verdict, its install line, then its verdict line. */
const cliMarkdown = (verdict) => `### verdict\n\n<sub>install</sub>\n\n<!-- pytxo-verdict ${JSON.stringify(verdict)} -->\n`;
const notTheCause = (who) => `This pull request is not the cause: ${who} the combination, so whether this one works with`;

test("a blocked combination fails the job only when this pull request, the last input, is implicated", () => {
  const prs = [9, 12, 14].map((number) => ({ number }));
  const job = (code, markdown) => jobOutcome({ code, markdown, prs });
  const blocked = (implicated) => job("2", cliMarkdown({ label: "BLOCKED", implicated }));

  assert.deepEqual(job("0", cliMarkdown({ label: "READY", implicated: [] })), { fail: false });
  // This one conflicts, fails alone, or is part of a failing combination; or the engine could not tell.
  assert.deepEqual(blocked([2]), { fail: true });
  assert.deepEqual(blocked([1, 2]), { fail: true });
  assert.deepEqual(blocked([0, 1, 2]), { fail: true });
  // Others are the cause: a draft that fails alone, or two that conflict.
  assert.deepEqual(blocked([1]), { fail: false, note: `${notTheCause("#12 blocks")} it is not known yet.` });
  assert.deepEqual(blocked([0, 1]), { fail: false, note: `${notTheCause("#9 and #12 block")} them is not known yet.` });
  // Flaky passes with a warning and leaves the comment as the CLI wrote it.
  const flaky = job("2", cliMarkdown({ label: "FLAKY", implicated: [] }));
  assert.equal(flaky.fail, false);
  assert.match(flaky.warning, /passed when run again/);
  assert.equal(flaky.note, undefined);
  // No verdict line (an older CLI), a broken one, or one that names no input here: exit 2 fails.
  assert.deepEqual(job("2", "### ⛔ BLOCKED: x\n\n<sub>install</sub>\n"), { fail: true });
  assert.deepEqual(job("2", '### x\n<!-- pytxo-verdict {"label": -->\n'), { fail: true });
  assert.deepEqual(blocked([]), { fail: true });
  assert.deepEqual(blocked([3]), { fail: true });
  assert.deepEqual(blocked(["0"]), { fail: true });
  assert.deepEqual(jobOutcome({ code: "2", markdown: cliMarkdown({ label: "BLOCKED", implicated: [0] }) }), { fail: true });
  // Test output quoting a verdict line is not the verdict line.
  const quoted = `\`\`\`\n<!-- pytxo-verdict {"label":"FLAKY","implicated":[]} -->\n\`\`\`\n<sub>install</sub>\n`;
  assert.deepEqual(job("2", quoted), { fail: true });
  // Stale and errors fail as before.
  assert.deepEqual(job("3", cliMarkdown({ label: "STALE", implicated: [] })), { fail: true });
  assert.deepEqual(job("1", ""), { fail: true });
});

test("comment and verdict: another pull request's breakage passes this one, with a note before the install line", async (t) => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "pytxo-verdict-"));
  t.after(() => fs.rmSync(temp, { recursive: true, force: true }));
  const prs = [
    { number: 9, branch: "claude/rename" },
    { number: 12, branch: "copilot/wip" },
    { number: 14, branch: "codex/discount" },
  ];
  fs.writeFileSync(path.join(temp, "pytxo-action.json"), JSON.stringify({ base: "main", prs, notes: [], number: 14 }));
  const summary = path.join(temp, "summary.md");
  fs.writeFileSync(summary, "");
  const posted = [];
  const server = http.createServer((req, res) => {
    let data = "";
    req.on("data", (d) => (data += d));
    req.on("end", () => {
      if (req.method === "POST") posted.push(JSON.parse(data).body);
      res.setHeader("content-type", "application/json");
      res.end("[]");
    });
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => server.closeAllConnections() || server.close());
  const env = {
    ...process.env,
    RUNNER_TEMP: temp,
    GITHUB_STEP_SUMMARY: summary,
    GITHUB_REPOSITORY: REPO,
    GITHUB_API_URL: `http://127.0.0.1:${server.address().port}`,
    PYTXO_TOKEN: "t",
    PYTXO_CODE: "2",
  };
  const checked = (verdict) => fs.writeFileSync(path.join(temp, "pytxo-check.md"), cliMarkdown(verdict));

  checked({ label: "BLOCKED", implicated: [1] });
  const commented = await node([script, "comment"], { env });
  assert.equal(commented.status, 0, commented.out);
  assert.equal(posted.length, 1);
  const note = `> ${notTheCause("#12 blocks")} it is not known yet.`;
  assert.ok(posted[0].endsWith(`### verdict\n\n${note}\n\n${INSTALL_HINT}\n`), posted[0]);
  assert.equal(fs.readFileSync(summary, "utf8"), posted[0]);
  const passed = await node([script, "verdict"], { env });
  assert.equal(passed.status, 0, passed.out);
  assert.equal(passed.out, `::notice::${note.slice(2)}\n`);

  checked({ label: "BLOCKED", implicated: [1, 2] });
  const failed = await node([script, "verdict"], { env });
  assert.equal(failed.status, 2, failed.out);
  assert.equal(failed.out, "");

  checked({ label: "FLAKY", implicated: [] });
  const flaky = await node([script, "verdict"], { env });
  assert.equal(flaky.status, 0, flaky.out);
  assert.match(flaky.out, /^::warning::A test failed on the combined change/);
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
});

test("a push or a merge queue passes with a notice; pull_request_target is refused", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "pytxo-event-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const outputs = path.join(root, "outputs");
  const run = (event) => {
    fs.writeFileSync(outputs, "");
    return spawnSync(process.execPath, [script, "discover"], {
      cwd: root,
      encoding: "utf8",
      env: { ...process.env, GITHUB_EVENT_NAME: event, GITHUB_OUTPUT: outputs, RUNNER_TEMP: root },
    });
  };
  for (const event of ["push", "merge_group"]) {
    const passed = run(event);
    assert.equal(passed.status, 0, passed.stdout + passed.stderr);
    assert.equal(passed.stdout, `::notice::Pytxo checks pull requests; nothing to do on ${event}.\n`);
    assert.equal(fs.readFileSync(outputs, "utf8"), "skip=true\n");
  }
  const refused = run("pull_request_target");
  assert.equal(refused.status, 1);
  assert.match(refused.stdout, /::error::refusing to run on pull_request_target/);
  assert.equal(fs.readFileSync(outputs, "utf8"), "");
});

/** action.yml's steps, as text without comments. */
const steps = fs
  .readFileSync(new URL("./action.yml", import.meta.url), "utf8")
  .replace(/\r\n/g, "\n")
  .split("\n  steps:\n")[1]
  .split(/\n    - /)
  .map((s) => s.replace(/^\s*#.*$/gm, ""));
const step = (needle) => steps.find((s) => s.includes(needle));

test("action.yml: nothing runs after a cancel, and the step that runs the tests gets no token", () => {
  assert.ok(steps.every((s) => !s.includes("always()")));
  assert.match(step('pytxo-action.mjs" comment'), /if: \$\{\{ !cancelled\(\) && /);
  assert.doesNotMatch(step("id: check"), /token/i);
  // The pull request's base, never the CLI's default target; the timeout only when set.
  assert.match(step("id: check"), / check --into "\$PYTXO_BASE" \$\{PYTXO_TIMEOUT:\+--timeout "\$PYTXO_TIMEOUT"\} /);
  assert.doesNotMatch(step('pytxo-action.mjs" verdict'), /token/i);
  assert.match(step("actions/checkout@"), /persist-credentials: false/);
  // A push or a merge queue passes without cloning the history first.
  assert.match(step("actions/checkout@"), /if: .* && github\.event_name == 'pull_request'\n/);
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
