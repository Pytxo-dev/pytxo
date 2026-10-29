import assert from "node:assert/strict";
import { spawn, spawnSync } from "node:child_process";
import fs from "node:fs";
import http from "node:http";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  approvedCommits,
  commentBody,
  DEFAULT_PATTERNS,
  glob,
  heldPrs,
  INSTALL_HINT,
  isAgentPr,
  jobOutcome,
  licenseGate,
  LICENSING,
  localName,
  MARKER,
  MAX_PRS,
  parsePatterns,
  selectPrs,
  upsertComment,
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
  assert.deepEqual(DEFAULT_PATTERNS, ["claude/*", "codex/*", "copilot/*", "cursor/*", "devin/*", "jules/*", "agent/*"]);
  assert.equal(parsePatterns(""), DEFAULT_PATTERNS);
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

test("a PR GitHub holds for workflow approval, with no run anyone let start, joins only its own run", () => {
  const copilot = pr(10, "copilot/fix-tax", { login: "Copilot" });
  const open = [
    pr(9, "claude/rename"),
    copilot,
    pr(11, "add-docs", { login: "copilot-swe-agent[bot]" }),
    pr(12, "agent/bump-deps", { login: "github-actions[bot]" }),
    pr(14, "codex/discount"),
  ];
  const other = selectPrs(open[4], open);
  assert.deepEqual(
    other.prs.map((p) => p.number),
    [9, 14],
  );
  assert.deepEqual(other.notes, ["3 agent pull requests that GitHub holds for workflow approval left out."]);
  // Its own run, once approved, checks it with the others.
  const own = selectPrs(copilot, open);
  assert.deepEqual(
    own.prs.map((p) => p.number),
    [9, 14, 10],
  );
  assert.deepEqual(own.notes, ["2 agent pull requests that GitHub holds for workflow approval left out."]);
  assert.deepEqual(selectPrs(open[4], [open[0], copilot, open[4]]).notes, ["1 agent pull request that GitHub holds for workflow approval left out."]);
});

/**
 * GitHub's answer to `url`: the open `pulls`, a branch's workflow `runs`, or
 * a pull request's `commits` (by number, oldest first). Anything else is [].
 */
function answer(url, { pulls = [], runs = {}, commits = {} }) {
  const { pathname, searchParams } = new URL(url, "http://localhost");
  if (pathname === `/repos/${REPO}/pulls`) return pulls;
  if (pathname === `/repos/${REPO}/actions/runs`) return { workflow_runs: runs[searchParams.get("branch")] ?? [] };
  const number = new RegExp(`^/repos/${REPO}/pulls/(\\d+)/commits$`).exec(pathname)?.[1];
  return number ? (commits[number] ?? []).map((sha) => ({ sha })) : [];
}

/** A pull_request workflow run on `sha`, from this repository unless `fork`. */
const run = (sha, attempt, conclusion, { fork = false } = {}) => ({
  event: "pull_request",
  head_sha: sha,
  run_attempt: attempt,
  conclusion,
  head_repository: { full_name: fork ? "someone/shop" : REPO },
});

/**
 * Asks a fake GitHub about the held PRs among `open` for `current`'s run;
 * `calls` are the paths asked. `refuse(path, n)` can turn the nth call into an
 * error, `{ status, headers, message }`, or into no connection, an Error.
 */
async function lookUp(current, open, data, { refuse = () => undefined } = {}) {
  const calls = [];
  const fetchImpl = async (url) => {
    const path = url.replace("https://api", "");
    calls.push(path);
    const error = refuse(path, calls.length);
    if (error instanceof Error) throw error;
    if (error) return { ok: false, status: error.status, headers: new Headers(error.headers), json: async () => ({ message: error.message ?? "" }) };
    return { ok: true, status: 200, headers: new Headers(), json: async () => answer(url, data) };
  };
  const found = await approvedCommits(fetchImpl, { apiUrl: "https://api", repo: REPO, token: "t", prs: heldPrs(current, open) });
  return { ...found, calls };
}

const [A, B, C] = ["a", "b", "c"].map((x) => x.repeat(40));

test("a held PR joins another PR's run at the newest of its commits whose run someone let start", async () => {
  const copilot = pr(12, "copilot/fix-tax", { login: "Copilot" });
  const current = pr(14, "codex/discount");
  const open = [pr(9, "claude/rename"), copilot, current];
  const { approved, notes, calls } = await lookUp(current, open, {
    runs: {
      "copilot/fix-tax": [
        run(C, 1, "action_required"), // the newest push, still held
        run(C, 2, "success", { fork: true }), // a fork's branch of the same name
        run("e".repeat(40), 2, "success"), // a commit a force-push took out of the PR
        run(B, 2, "failure"), // approved: attempt 2 started, whatever it concluded
        run(A, 2, "success"),
      ],
    },
    commits: { 12: [A, B, C] },
  });
  assert.deepEqual([...approved], [[12, B]]);
  assert.deepEqual(notes, []);
  assert.deepEqual(calls, [
    "/repos/acme/shop/actions/runs?event=pull_request&branch=copilot%2Ffix-tax&per_page=100",
    "/repos/acme/shop/pulls/12/commits?per_page=100",
  ]);
  const { prs, notes: left } = selectPrs(current, open, DEFAULT_PATTERNS, MAX_PRS, approved);
  assert.deepEqual(
    prs.map((p) => p.number),
    [9, 12, 14],
  );
  assert.deepEqual(left, []);
});

test("with approval turned off, a held PR joins at a commit whose first run succeeded", async () => {
  const current = pr(14, "codex/discount");
  const open = [pr(12, "agent/bump-deps", { login: "github-actions[bot]" }), current];
  const { approved } = await lookUp(current, open, { runs: { "agent/bump-deps": [run(B, 1, "success")] }, commits: { 12: [A, B] } });
  assert.deepEqual([...approved], [[12, B]]);
});

test("a held PR whose runs wait for approval, or waited until GitHub failed them, stays out after one call", async () => {
  const current = pr(14, "codex/discount");
  const open = [pr(9, "claude/rename"), pr(12, "copilot/fix-tax", { login: "Copilot" }), current];
  // A run nobody approves for 30 days ends as a failure at attempt 1.
  const runs = { "copilot/fix-tax": [run(B, 1, "action_required"), run(A, 1, "failure")] };
  const { approved, notes, calls } = await lookUp(current, open, { runs, commits: { 12: [A, B] } });
  assert.equal(approved.size, 0);
  assert.deepEqual(notes, []);
  assert.equal(calls.length, 1);
  const { prs, notes: left } = selectPrs(current, open, DEFAULT_PATTERNS, MAX_PRS, approved);
  assert.deepEqual(
    prs.map((p) => p.number),
    [9, 14],
  );
  assert.deepEqual(left, ["1 agent pull request that GitHub holds for workflow approval left out."]);
});

test("GitHub is asked about the newest 9 held PRs, and without actions: read they stay out with a note", async () => {
  const current = pr(30, "codex/discount");
  const held = Array.from({ length: 12 }, (_, i) => pr(i + 1, `copilot/b${i + 1}`, { login: "Copilot" }));
  assert.deepEqual(
    heldPrs(current, [...held, current]).map((p) => p.number),
    [4, 5, 6, 7, 8, 9, 10, 11, 12],
  );
  for (const status of [403, 404]) {
    const refuse = () => ({ status, message: "Resource not accessible by integration" });
    const { approved, notes, calls } = await lookUp(current, [...held, current], {}, { refuse });
    assert.equal(approved.size, 0);
    assert.deepEqual(notes, ["Add `actions: read` to the workflow's permissions to check Copilot's pull requests once someone lets their run start."]);
    // The first refusal ends the asking.
    assert.equal(calls.length, 1);
  }
});

/** Two held PRs, each with an approved run, for #14's run. */
const twoHeld = () => ({
  current: pr(14, "codex/discount"),
  open: [pr(12, "copilot/fix-tax", { login: "Copilot" }), pr(13, "copilot/docs", { login: "Copilot" }), pr(14, "codex/discount")],
  data: { runs: { "copilot/fix-tax": [run(A, 2, "success")], "copilot/docs": [run(B, 2, "success")] }, commits: { 12: [A], 13: [B] } },
});

test("any other error on either call leaves the held PRs not yet let in out, says why, and the run goes on", async () => {
  const { current, open, data } = twoHeld();
  const on = (call, error) => (path) => (path.includes(call) ? error : undefined);
  for (const [refuse, why, asked] of [
    // The runs call answered, so the token has actions: read: this 403 is something else.
    [on("/commits", { status: 403 }), " (HTTP 403)", 2],
    [on("/actions/runs", { status: 429 }), " (HTTP 429)", 1],
    [on("/actions/runs", { status: 502 }), " (HTTP 502)", 1],
    [on("/actions/runs", new TypeError("fetch failed")), "", 1],
  ]) {
    const { approved, notes, calls } = await lookUp(current, open, data, { refuse });
    assert.equal(approved.size, 0);
    assert.deepEqual(notes, [`#12, #13 left out: GitHub did not answer${why}.`]);
    assert.equal(calls.length, asked);
  }
});

test("a rate limit GitHub answers with 403 is not a missing permission, and a held PR already let in stays in", async () => {
  const { current, open, data } = twoHeld();
  // Either sign of a rate limit on its own: the header, or the message.
  for (const limit of [
    { status: 403, headers: { "x-ratelimit-remaining": "0" } },
    { status: 403, message: "You have exceeded a secondary rate limit. Please wait a few minutes before you try again." },
  ]) {
    // #12's runs and commits are answered; the limit hits the third call, #13's runs.
    const { approved, notes } = await lookUp(current, open, data, { refuse: (path, n) => (n === 3 ? limit : undefined) });
    assert.deepEqual([...approved], [[12, A]]);
    assert.deepEqual(notes, ["#13 left out: GitHub did not answer (HTTP 403)."]);
  }
});

test("nothing to check: not an agent PR, or the only one", () => {
  assert.match(selectPrs(pr(3, "matt/readme"), []).reason, /#3 is not an agent pull request/);
  const alone = selectPrs(pr(4, "claude/a"), [pr(4, "claude/a"), pr(5, "matt/b")]);
  assert.match(alone.reason, /only agent pull request into main this run can check/);
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
  // A held PR names the commit someone let run, which can be older than its newest push.
  const held = { number: 12, branch: "copilot/fix-tax", sha: `abc1234${"0".repeat(33)}`, held: true };
  const atCommit = commentBody({ prs: [held, prs[1]], markdown: cli, stderr: "", code: "2" });
  assert.ok(atCommit.includes("\nChecked together: #12 `copilot/fix-tax` at abc1234 · #14 `codex/discount`\n"), atCommit);
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

test("discover fetches the agent PRs as local branches and the base, a held one at the commit someone let run", async (t) => {
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
  // #14 gets a push after the run started: the run checks the commit it saw.
  sh(work, "checkout", "-q", "codex/discount");
  fs.writeFileSync(path.join(work, "late.txt"), "late\n");
  sh(work, "add", "-A");
  sh(work, "commit", "-q", "-m", "pushed during the run");
  const moved = sh(work, "rev-parse", "HEAD");
  // #12 is Copilot's: someone let the run on its first commit start; its newer push still waits.
  sh(work, "checkout", "-q", "-b", "copilot/fix-tax", "main");
  const copilot = [];
  for (const step of ["approved", "waiting"]) {
    fs.writeFileSync(path.join(work, "12.txt"), `${step}\n`);
    sh(work, "add", "-A");
    sh(work, "commit", "-q", "-m", step);
    copilot.push(sh(work, "rev-parse", "HEAD"));
  }
  sh(work, "checkout", "-q", "main");
  const origin = path.join(root, "origin.git");
  sh(root, "clone", "-q", "--bare", work, origin);
  sh(origin, "update-ref", "refs/pull/9/head", heads[9]);
  sh(origin, "update-ref", "refs/pull/12/head", copilot[1]);
  sh(origin, "update-ref", "refs/pull/14/head", moved);
  sh(origin, "branch", "-D", "claude/rename", "codex/discount", "copilot/fix-tax");
  const clone = path.join(root, "clone");
  sh(root, "clone", "-q", "--depth", "1", `file://${origin.replace(/\\/g, "/")}`, clone);
  sh(clone, "checkout", "-q", "--detach");

  const pulls = [
    pr(9, "claude/rename"),
    pr(14, "codex/discount"),
    pr(11, "matt/readme"),
    pr(12, "copilot/fix-tax", { login: "Copilot" }),
    pr(13, "copilot/docs", { login: "Copilot" }),
  ];
  pulls[0].head.sha = heads[9];
  pulls[1].head.sha = heads[14];
  pulls[3].head.sha = copilot[1];
  // #13's approved commit is gone from the origin, as after a force-push.
  const gone = "d".repeat(40);
  const github = {
    pulls,
    runs: {
      "copilot/fix-tax": [run(copilot[1], 1, "action_required"), run(copilot[0], 2, "success")],
      "copilot/docs": [run(gone, 2, "success")],
    },
    commits: { 12: copilot, 13: [gone] },
  };
  const server = http.createServer((req, res) => {
    assert.equal(req.headers.authorization, "Bearer secret");
    res.setHeader("content-type", "application/json");
    res.end(JSON.stringify(answer(req.url, github)));
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => server.closeAllConnections() || server.close());

  const event = path.join(root, "event.json");
  fs.writeFileSync(event, JSON.stringify({ pull_request: pulls[1] }));
  const outputs = path.join(root, "outputs");
  fs.writeFileSync(outputs, "");
  const discover = () =>
    node([script, "discover"], {
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
  const discovered = await discover();
  assert.equal(discovered.status, 0, discovered.out);
  assert.match(fs.readFileSync(outputs, "utf8"), /skip=false\nbase=main\n/);
  assert.equal(fs.readFileSync(path.join(root, "pytxo-branches.txt"), "utf8"), "claude/rename\ncopilot/fix-tax\ncodex/discount\n");
  assert.equal(sh(clone, "rev-parse", "refs/heads/claude/rename"), heads[9]);
  // Copilot's PR at the commit someone let run, not at its newer push.
  assert.equal(sh(clone, "rev-parse", "refs/heads/copilot/fix-tax"), copilot[0]);
  assert.equal(sh(clone, "rev-parse", "refs/heads/codex/discount"), heads[14]);
  assert.equal(sh(clone, "rev-parse", "--is-shallow-repository"), "false");
  sh(clone, "merge-base", "main", "codex/discount");
  const state = JSON.parse(fs.readFileSync(path.join(root, "pytxo-action.json"), "utf8"));
  assert.deepEqual(state.prs, [
    { number: 9, sha: heads[9], branch: "claude/rename" },
    { number: 12, sha: copilot[0], branch: "copilot/fix-tax", held: true },
    { number: 14, sha: heads[14], branch: "codex/discount" },
  ]);
  assert.deepEqual(state.notes, ["#13 left out: its commit ddddddd could not be fetched."]);

  // With #13 the only other one, nothing is left to combine.
  github.pulls = [pulls[1], pulls[4]];
  fs.writeFileSync(outputs, "");
  const lone = await discover();
  assert.equal(lone.status, 0, lone.out);
  assert.equal(fs.readFileSync(outputs, "utf8"), "skip=true\n");
  const after = JSON.parse(fs.readFileSync(path.join(root, "pytxo-action.json"), "utf8"));
  assert.deepEqual(after.prs, []);
  assert.match(after.reason, /^#14 is the only agent pull request into main this run can check/);
  assert.deepEqual(after.notes, state.notes);
});

test("discover deletes a tag named like a held PR's branch, so the CLI checks the approved commit, and leaves out a name git reads as something else", async (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "pytxo-names-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  const work = path.join(root, "work");
  fs.mkdirSync(work);
  sh(work, "init", "-q", "-b", "main");
  sh(work, "config", "user.email", "t@t");
  sh(work, "config", "user.name", "t");
  const commit = (file, text) => {
    fs.writeFileSync(path.join(work, file), `${text}\n`);
    sh(work, "add", "-A");
    sh(work, "commit", "-q", "-m", text);
    return sh(work, "rev-parse", "HEAD");
  };
  commit("a.txt", "base");
  sh(work, "checkout", "-q", "-b", "codex/discount", "main");
  const discount = commit("14.txt", "14");
  // #13's branch is called FETCH_HEAD, the file git's fetches write in .git.
  sh(work, "checkout", "-q", "-b", "thirteen", "main");
  const thirteen = commit("13.txt", "13");
  sh(work, "checkout", "-q", "-b", "copilot/fix-tax", "main");
  const copilot = [commit("12.txt", "approved"), commit("12.txt", "never approved")];
  sh(work, "checkout", "-q", "main");
  // Anyone who can push a tag: one named like Copilot's branch, at its unapproved commit.
  sh(work, "tag", "copilot/fix-tax", copilot[1]);
  const origin = path.join(root, "origin.git");
  sh(root, "clone", "-q", "--bare", work, origin);
  sh(origin, "update-ref", "refs/pull/12/head", copilot[1]);
  sh(origin, "update-ref", "refs/pull/13/head", thirteen);
  sh(origin, "update-ref", "refs/pull/14/head", discount);
  // As actions/checkout with fetch-depth: 0 leaves it: every branch and every tag.
  const clone = path.join(root, "clone");
  sh(root, "clone", "-q", `file://${origin.replace(/\\/g, "/")}`, clone);
  sh(clone, "checkout", "-q", "--detach");

  const pulls = [pr(12, "copilot/fix-tax", { login: "Copilot" }), pr(13, "FETCH_HEAD", { login: "claude[bot]" }), pr(14, "codex/discount")];
  [copilot[1], thirteen, discount].forEach((sha, i) => (pulls[i].head.sha = sha));
  const github = {
    pulls,
    runs: { "copilot/fix-tax": [run(copilot[1], 1, "action_required"), run(copilot[0], 2, "success")] },
    commits: { 12: copilot },
  };
  const server = http.createServer((req, res) => {
    res.setHeader("content-type", "application/json");
    res.end(JSON.stringify(answer(req.url, github)));
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  t.after(() => server.closeAllConnections() || server.close());
  const event = path.join(root, "event.json");
  const outputs = path.join(root, "outputs");
  const discover = (current) => {
    fs.writeFileSync(event, JSON.stringify({ pull_request: current }));
    fs.writeFileSync(outputs, "");
    return node([script, "discover"], {
      cwd: clone,
      env: {
        ...process.env,
        GITHUB_EVENT_NAME: "pull_request",
        GITHUB_EVENT_PATH: event,
        GITHUB_OUTPUT: outputs,
        GITHUB_REPOSITORY: REPO,
        GITHUB_API_URL: `http://127.0.0.1:${server.address().port}`,
        RUNNER_TEMP: root,
        PYTXO_TOKEN: "t",
      },
    });
  };
  const state = () => JSON.parse(fs.readFileSync(path.join(root, "pytxo-action.json"), "utf8"));

  const discovered = await discover(pulls[2]);
  assert.equal(discovered.status, 0, discovered.out);
  assert.equal(fs.readFileSync(path.join(root, "pytxo-branches.txt"), "utf8"), "copilot/fix-tax\ncodex/discount\n");
  // The CLI's own lookup now gives the commit someone let run, and the tag is gone from this checkout.
  assert.equal(sh(clone, "rev-parse", "--verify", "--quiet", "copilot/fix-tax^{commit}"), copilot[0]);
  assert.equal(sh(clone, "tag", "--list", "copilot/fix-tax"), "");
  assert.deepEqual(state().prs, [
    { number: 12, sha: copilot[0], branch: "copilot/fix-tax", held: true },
    { number: 14, sha: discount, branch: "codex/discount" },
  ]);
  // FETCH_HEAD reads the last fetch, not refs/heads/FETCH_HEAD.
  assert.deepEqual(state().notes, ["#13 left out: its branch name `FETCH_HEAD` already means something else to git."]);

  // When that is this run's own pull request, there is nothing it can check.
  const own = await discover(pulls[1]);
  assert.equal(own.status, 0, own.out);
  assert.equal(fs.readFileSync(outputs, "utf8"), "skip=true\n");
  assert.deepEqual(state().prs, []);
  assert.equal(state().reason, "#13 cannot be checked: its branch name `FETCH_HEAD` already means something else to git.");
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
  // The `check` input reaches the CLI as its PYTXO_CHECK setting.
  assert.match(step("id: check"), /PYTXO_CHECK: \$\{\{ inputs\.check \}\}/);
  assert.doesNotMatch(step('pytxo-action.mjs" verdict'), /token/i);
  assert.match(step("actions/checkout@"), /persist-credentials: false/);
  // A push or a merge queue passes without cloning the history first.
  assert.match(step("actions/checkout@"), /if: .* && github\.event_name == 'pull_request'\n/);
});

/** Dodo Payments giving `answers` in turn (an Error is no connection); `calls` holds what was sent. */
function fakeDodo(...answers) {
  const calls = [];
  const fetchImpl = async (url, init) => {
    calls.push({ url, ...init, body: JSON.parse(init.body) });
    const answer = answers.shift();
    if (answer instanceof Error) throw answer;
    return { ok: answer.status < 300, status: answer.status, json: async () => answer.body };
  };
  return { calls, fetchImpl };
}

const KEY = "PYTXO-PRO-SECRET-0001";
/** The gate switched on for a private repository whose trial ended on 2026-10-15. */
const gate = (fetchImpl, opts = {}) =>
  licenseGate({
    licensing: true,
    isPrivate: true,
    key: KEY,
    trialStart: () => new Date("2026-10-01T00:00:00Z"),
    now: new Date("2026-11-20T00:00:00Z"),
    fetchImpl,
    ...opts,
  });

test("Pro licensing is switched off: no key is checked and nothing is sent", async () => {
  assert.equal(LICENSING, false);
  const dodo = fakeDodo();
  const off = await licenseGate({ isPrivate: true, key: KEY, trialStart: () => new Date(0), now: new Date(), fetchImpl: dodo.fetchImpl });
  assert.deepEqual(off, { allowed: true });
  assert.deepEqual(dodo.calls, []);
});

test("after the 14-day trial a private repository runs with a key Dodo Payments says is valid, and sends only the key", async () => {
  const dodo = fakeDodo({ status: 200, body: { valid: true } });
  assert.deepEqual(await gate(dodo.fetchImpl), { allowed: true });
  assert.equal(dodo.calls.length, 1);
  const [call] = dodo.calls;
  assert.equal(call.url, "https://live.dodopayments.com/licenses/validate");
  assert.equal(call.method, "POST");
  assert.deepEqual(call.body, { license_key: KEY });
  // No GitHub token, nor anything else, goes with it.
  assert.deepEqual(call.headers, { "content-type": "application/json" });
});

test("after the trial a key that is not valid is refused, and no answer from Dodo Payments runs the check anyway", async () => {
  const results = [];
  // A lapsed subscription or a mistyped key; 422 is Dodo's answer to a request it cannot read.
  for (const answer of [{ status: 200, body: { valid: false } }, { status: 422, body: {} }]) {
    const refused = await gate(fakeDodo(answer).fetchImpl);
    assert.equal(refused.allowed, false);
    assert.match(refused.notice, /^The 14-day trial for private repositories is over \(the license key is not valid\)\. Add .* PYTXO_LICENSE /);
    results.push(refused);
  }
  for (const [answer, why] of [
    [new TypeError("fetch failed"), "no answer"],
    [{ status: 429 }, "HTTP 429"],
    [{ status: 503 }, "HTTP 503"],
  ]) {
    const ran = await gate(fakeDodo(answer).fetchImpl);
    assert.deepEqual(ran, { allowed: true, warning: `Dodo Payments could not check the Pytxo Pro license key (${why}), so the check runs anyway.` });
    results.push(ran);
  }
  assert.ok(results.every((r) => !JSON.stringify(r).includes(KEY)));
});

test("Dodo Payments is asked nothing for a public repository, during the trial, or without a key", async () => {
  const dodo = fakeDodo();
  const inTrial = () => new Date("2026-11-10T00:00:00Z");
  assert.deepEqual(await gate(dodo.fetchImpl, { isPrivate: false }), { allowed: true });
  const trial = await gate(dodo.fetchImpl, { key: "", trialStart: inTrial });
  assert.equal(trial.allowed, true);
  assert.match(trial.notice, /: 4 days left\. https:\/\/pytxo\.com\/plans$/);
  // A key set during the trial is checked once the trial ends, with no reminder before.
  assert.deepEqual(await gate(dodo.fetchImpl, { trialStart: inTrial }), { allowed: true });
  const none = await gate(dodo.fetchImpl, { key: "" });
  assert.equal(none.allowed, false);
  assert.match(none.notice, /is over\. Add a Pytxo Pro license key as the PYTXO_LICENSE secret/);
  assert.deepEqual(dodo.calls, []);
});
