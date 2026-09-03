# Pytxo v1.2 commit-boundary demo

For the 75-second launch-film treatment, generated bridge prompts, narration,
gallery frames, and recording rules, see [`GROK_DEMO_BRIEF.md`](GROK_DEMO_BRIEF.md).

This is a rehearsable 2–4 minute release demo of Pytxo's real differentiated
value: an autonomous worker does not get a direct path from intent to repository
mutation. Pytxo isolates the work, verifies it, shows the exact prepared bytes,
requires human Apply, and records the observed outcome.

The demo uses a clearly disclosed deterministic local adapter. That removes
provider latency and model variance while exercising the same runner,
isolated-workspace, verification, review-package, Apply, and ledger code as a headless
agent CLI. It is evidence for the control boundary, not a claim about model
intelligence.

## Preflight

Use a clean 1600×1000 or 1920×1080 desktop at 100% scaling. Hide notifications,
account details, terminal history, and unrelated repositories. Build the exact
candidate to be recorded:

```powershell
Set-Location C:\pytxo
cargo build -p pytxo-cli --release
Push-Location apps\desktop
npm ci
npm run build:native
Pop-Location
$env:PYTXO_DEMO_WORKSPACE = & .\tooling\demos\commit-boundary\prepare.ps1 | Select-Object -Last 1
Set-Location $env:PYTXO_DEMO_WORKSPACE
& C:\pytxo\target\release\pytxo.exe trust orbit
Start-Process C:\pytxo\target\release\pytxo-desktop.exe
```

Expected result: baseline `npm test` passes, the final preparation line is a
new path under `C:\pytxo\target\release-demo\`, `PYTXO_HOME` points to a fresh
adjacent demo-only catalog, `WEBVIEW2_USER_DATA_FOLDER` points inside that same
demo home, and `trust orbit` confirms the folder boundary. The Desktop process
inherits both isolated stores, so no personal workspace, run, approval, or
persisted UI recent can enter the recording.

Run one full rehearsal and preserve its completed run. Do not record a take if
the baseline, verification command, review package, or Apply evidence is
missing.

## Main sequence (about 2:45)

### 00:00–00:20 — Problem and promise

**Shot:** Pytxo Desktop on Work, full frame. Move once to the **Commit boundary**
panel; do not begin on a terminal wall.

**Narration:**

> An autonomous coding tool can turn a sentence into repository changes before
> you understand the effect. Pytxo sits at that commit boundary: isolate the
> work, verify the result, show the exact bytes, and only then let a person Apply.

The first 20 seconds must visibly include the words **Commit boundary** and
**Apply**.

### 00:20–00:50 — Create controlled work

**Shot:** Terminal, cropped to the demo workspace. State aloud that the adapter
is deterministic. Run:

```powershell
& C:\pytxo\target\release\pytxo.exe run `
  --repo $env:PYTXO_DEMO_WORKSPACE `
  --config (Join-Path $env:PYTXO_DEMO_WORKSPACE "pytxo.toml") `
  --agents 1 `
  --execution subprocess `
  --cmd "node demo-agent.mjs"
```

**Expected visible result:** one Orbit task runs outside the primary checkout in
the platform's reported worktree or overlay backend,
`npm test` reports three passing tests, and Pytxo reports a terminal run rather
than success before verification.

**Narration:**

> This local adapter is deterministic so the demo cannot be rescued by a lucky
> model response. Pytxo still gives it a real isolated workspace, an exact path
> claim, and a real verification command.

### 00:50–01:30 — Pytxo intervention and evidence

**Exact actions:**

1. Return to Desktop and select the new `commit-boundary-*` workspace in the
   title-bar switcher.
2. Open **Work** and select the newest run.
3. Hold on **Waves** until it reads `1 of 1 settled · 1 passed`.
4. In **Commit boundary**, point to the effective Orbit profile, workspace
   isolation, Apply boundary, prepared package digest, and verification state.
5. Open **Run Review**.

**Expected visible result:** the run and receipt belong to the selected
workspace; the review lists only `README.md`, `src/risk-policy.mjs`, and
`test/risk-policy.test.mjs`; additions and deletions are inspectable; no state
is styled as verified without mechanical evidence.

**Narration:**

> The adapter's claim is not the proof. Pytxo records what profile actually ran,
> where the work was contained, which paths changed, and whether the independent
> test command passed. The review is the stored candidate, not a fresh diff that
> can drift after approval.

### 01:30–02:15 — Human authorization and exact Apply

Before clicking Apply, create unrelated checkout drift to show that the guard is
path-specific:

```powershell
Set-Content -LiteralPath (Join-Path $env:PYTXO_DEMO_WORKSPACE "operator-note.txt") -Value "unrelated operator note"
```

**Exact actions:**

1. In Run Review, select the prepared package and click **Apply** once.
2. Confirm the consequence in the dialog.
3. Wait for the committed result; do not cut from a loading state to success.
4. Return to the terminal and run:

```powershell
Set-Location $env:PYTXO_DEMO_WORKSPACE
git status --short
npm test
```

**Expected visible result:** Apply commits the three reviewed files despite the
unrelated `operator-note.txt`; the note remains untracked; all three tests pass
in the primary checkout. The receipt must distinguish committed from verified.

**Narration:**

> Apply rechecks every affected preimage and commits only the reviewed package.
> Unrelated operator work is left alone. Then we observe the post-state again:
> the exact files are present and the tests pass in the real checkout.

### 02:15–02:45 — Durable outcome

**Exact actions:** open **History**, select the run, and hold on the Apply attempt,
package digest, timestamps, and final evidence. End on Work with the commit
boundary visible.

**Narration:**

> History keeps the decision and outcome together: proposed, verified,
> authorized, committed, and observed. If Pytxo cannot prove recovery, it says
> recovery required. That honesty—not another agent dashboard—is the product.

End card: **Intent is not a commit. Pytxo makes the boundary explicit.**

## Controlled failure insert (optional 25–35 seconds)

Prepare a fresh session and run it through review, but before Apply modify
`src/risk-policy.mjs` in the primary checkout. Apply must refuse the stale
affected path; it must not report committed or verified. Show the stale state,
then stop the take. Do not repair or overwrite the operator change on camera.

This insert is valid only after a rehearsal proves the refusal and corresponding
ledger state on the exact release candidate.

### Verified rehearsal evidence

The main sequence passed through packaged Windows Desktop run
`54924278-5e91-40d0-a7de-6f8e99cde4a3` with package
`072f954fcaa27bc0c9d36e561139614223cac176e3871857ea1f423a4165f3b0`.
The three reviewed paths applied, unrelated `operator-note.txt` survived, the
primary checkout passed 3/3 tests, and History recorded one committed attempt.

The controlled failure passed through packaged run
`47da6a06-a3f3-4b66-8425-838efd8a7809` with package
`30b29840dc68225b7a521ce0b78ca144d7a65c8555bc71af39f781f784e086a9`.
Affected-path drift caused stale-review refusal; no other reviewed path applied,
baseline tests remained 2/2, and History recorded Apply failed without a success
receipt. The optional failure insert is therefore approved for this candidate.

Runtime captures are retained under `target/release-demo/` as
`final-native-work.png`, `final-native-review.png`,
`final-native-apply-confirm.png`, `final-native-applied.png`,
`final-native-history.png`, `final-native-stale-refusal.png`, and
`final-native-stale-history.png`.

## Fallback behavior

- If the run fails or verification is absent, keep the failure on screen long
  enough to read, stop recording, and prepare a new session. Never reuse its
  package as a success take.
- If Desktop does not show the new domain, reopen the title-bar workspace
  switcher once. If it is still absent, stop; do not substitute preview data.
- If Run Review lacks the three expected paths or the package digest, stop.
- If Apply reports stale unexpectedly, preserve the state as failure evidence,
  prepare a new session, and investigate before another take.
- If the Desktop recording fails after a successful rehearsal, reuse the same
  preserved run only while its primary checkout and prepared package remain
  unchanged.

## Recording checklist

- [ ] Exact release CLI and packaged Desktop are used.
- [ ] A new committed fixture session is prepared; baseline tests pass.
- [ ] Orbit trust is visible and no provider key or account token is shown.
- [ ] The deterministic adapter is disclosed in narration and on-screen notes.
- [ ] One continuous run ID connects Work, Run Review, Apply, and History.
- [ ] Receipt, package digest, three paths, and verification command are legible.
- [ ] Apply is clicked once and the actual settled state is shown.
- [ ] Primary-checkout `git status` and `npm test` are recorded after Apply.
- [ ] No preview backend, fabricated percentage, retired route, email, username,
      API key, absolute personal path, or notification appears.
- [ ] Capture is reviewed once at full size and once at 50% size.
- [ ] Optional failure insert is used only if it passed a same-build rehearsal.

## Suggested shots

Use hard cuts between Terminal, Work, Run Review, and History. Inside each view,
use at most a restrained 1.25–1.35× crop anchored to the evidence being named.
Hold package and Apply outcomes for at least two seconds. A terminal close-up is
supporting evidence, not the hero shot.

## Short release description

Pytxo is the commit layer for autonomous work. This demo shows a real isolated
repository task moving through independent verification, exact-byte review,
human Apply, and durable post-state evidence—without giving the worker a direct
path to the primary checkout.

## Short social description

An agent's claim is not a commit. Pytxo isolates the work, verifies the result,
shows the exact bytes, and records what actually happened after Apply.
