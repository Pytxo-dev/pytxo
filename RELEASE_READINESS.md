# Pytxo v1.2.2 release readiness

**Assessment date:** 2026-09-04
**Branch:** `codex/release-1.2.2-integrity`
**Baseline HEAD:** `9139a908e05cf796578c094d485ee45baf50ba0d` (`origin/main`)
**Recommendation:** **NOT READY — hosted corrective release and artifact audit pending**
**Publication state:** v1.2.2 passes the complete local gate; protected CI, fresh-tag publication, and post-publication verification remain required

## Executive assessment

Pytxo v1.2.2 is a corrective release-integrity patch over the fully tested
v1.2.1 product. v1.2.1 exercised the actual Windows-first commit boundary, not
only preview UI: a deterministic worker ran in a persisted ProjFS-backed
overlay, independent verification passed, Desktop displayed the exact stored
package and receipt, an explicit human confirmation gated Apply, unrelated
operator drift survived, the primary checkout passed post-state tests, and
History recorded the committed attempt. v1.2.2 changes version and publication
surfaces, not that runtime behavior.

A separate packaged rehearsal changed an affected checkout path after review.
Pytxo refused Apply, did not partially apply the other reviewed files, did not
show success, and recorded a failed attempt. This is the most important
adversarial proof for the product thesis: an agent's claim and a prepared package
do not become a commit when the preconditions are no longer true.

The fresh patch is not ready to claim until protected CI completes, the corrected
workflow publishes a new v1.2.2 tag, and independently downloaded public assets
match the private build and checksum manifests. The repository has a Tauri
updater-signing key, but no Windows Authenticode certificate or local Git signing
key is configured. Narrated media also remains conditional on licensed audio.

## What changed

### Architecture and product boundary

- Folder trust is an out-of-band ceiling across global, per-agent, project-root,
  stdin, MCP, and offline-entitlement paths.
- Cloud repository/cache/delta egress requires dual consent, a hash-bound
  manifest, protected-path and credential-content checks, no-follow file reads,
  and a separate opt-in before transport failure may fall back locally. Policy
  denial never falls back.
- Verifiers have a separate enforcement receipt, minimal environment, bounded
  output, profile/network/HITL gates, process-tree timeout, and fail-closed cloud
  behavior.
- Failed or verification-failed tasks cannot unlock dependents; failed durable
  runs return failure rather than a false success.
- Detached dispatch persists `starting` plus exact active ownership before it
  returns. Concurrent dispatch cannot replace another live owner, and crashed
  ownership is reconciled.
- Fleet nodes run concurrently within approved waves and unfinished rows settle
  failed/skipped on error.
- Apply packages remain one execution domain and one repository root. Stored
  blobs, path preimages, mutation leases, journal reconciliation, and recovery
  states are preserved rather than broadened into a universal transaction claim.

### Desktop and UX

- The v1.2 shell is Work, History, and Setup, with workspace switching in the
  title bar and approvals as an overlay.
- Run and agent rows carry their owning execution domain. Review, recovery, and
  History no longer substitute the currently selected workspace domain.
- Completed runs retain their agents/waves, and Desktop prefers the persisted
  isolation receipt over recomputed configuration intent.
- Partial native snapshots expose per-domain diagnostics rather than rendering
  omitted data as a verified empty state.
- Approval records carry causal domain/run/agent/request/action identity; the UI
  refuses recency-based substitute evidence.
- Apply now opens a focus-contained, Escape-cancellable confirmation naming the
  exact path count and package digest.
- Dark-theme semantic colors, compact layout, empty/loading/error/stale/success
  states, and Storybook catalog navigation were reconciled with the current IA.

### Reliability

- PTY process identity is persisted at spawn with Windows creation identity.
  Stop rejects PID reuse, kills descendants, confirms exit, and preserves the
  `cancelled` terminal state.
- Agent futures and wave siblings are supervised; lifecycle errors become
  durable failed results; Race/MCP cleanup is explicit; cloud teardown is RAII.
- SQLite upgrades are transactional and schema-verified before version advance.
- The final audit found a Windows-only race in the Link HTTP test harness:
  nonblocking mode could leak from the listener to accepted sockets under
  parallel load. Accepted sockets now restore blocking mode, read the complete
  request, send a complete response, half-close, and wait for client EOF.
- Ratatui is upgraded to 0.30.2 and resolves to patched `lru 0.18.3`; optional
  Linux FUSE resolves to patched `fuser 0.16.0`. The declared Rust minimum is
  now 1.88.

### Security

- Paddle entitlement changes require a configured secret, fresh signature,
  allowlisted price identity, durable event replay record, and persisted
  subscription/customer/user ownership. Duplicate delivery is idempotent.
- The duplicate web entitlement provisioner was removed in favor of a
  fail-closed Link proxy.
- Hosted Link and cloud-sandbox processes reject unauthenticated public binds
  and reject auth-enabled startup without credentials.
- Reusable Clerk bearer tokens are no longer placed in Desktop custom-protocol
  URLs. The unavailable v1.2 account-return flow is stated honestly.
- DeepSpace raw/high-fidelity MCP reads are denied/capped, and MCP error
  envelopes are sanitized without converting errors into success.
- CLI/npm installers require exact asset identity, SHA-256, embedded version,
  and atomic replacement. Release Actions are pinned to immutable revisions.

### Release publication integrity

- The v1.2.1 hosted workflow passed, but a later Desktop mirror stage inherited
  obsolete tracked root `dist/` files and replaced the public Windows CLI and
  five-platform checksum manifest. Those assets were restored and independently
  rehashed, but that tag's artifact history is now mutable.
- v1.2.2 removes and ignores repository-root release staging, empties every
  staging directory after checkout, and requires exact non-empty inventories
  immediately before private, public CLI, and public Desktop publication.
- Signed Desktop releases require one updater asset plus `latest.json`;
  unsigned releases require neither. Unit tests cover both valid cases, the
  original stale-file collision, empty assets, nested directories, and missing
  signed updater evidence.
- Version parity covers all 15 versioned Rust lock entries plus Desktop/demo
  lockfiles, Tauri, npm, web download metadata, installer copy, README, public
  changelog, and versioned release notes. These tests run in reusable CI.
- The release icon pipeline no longer uses abandoned `to-ico` or its legacy
  request/image dependency chain. A bounded PNG-in-ICO encoder is unit-tested;
  both generators pass and a clean tooling audit reports zero vulnerabilities.

## Runtime evidence

### Successful commit boundary

- Workspace: `target/release-demo/commit-boundary-20260902-122738-212`
- Run: `54924278-5e91-40d0-a7de-6f8e99cde4a3`
- Base commit: `74c284d50f9b99b5ceb636c61bd3a1cf82bfc78a`
- Package: `072f954fcaa27bc0c9d36e561139614223cac176e3871857ea1f423a4165f3b0`
- Work showed `1 of 1 settled · 1 passed`, the exact agent, Orbit, and
  `overlay · projfs-sparse-copy-v2` from the persisted receipt.
- Review contained exactly `README.md`, `src/risk-policy.mjs`, and
  `test/risk-policy.test.mjs`.
- The confirmation dialog appeared before mutation and named three paths plus
  the exact digest.
- Unrelated `operator-note.txt` remained untracked after Apply.
- The three reviewed paths were applied; primary-checkout tests passed 3/3.
- History showed Completed / Applied with one committed Apply attempt.
- Captures: `target/release-demo/final-native-work.png`,
  `final-native-review.png`, `final-native-apply-confirm.png`,
  `final-native-applied.png`, and `final-native-history.png`.

### Stale affected-path refusal

- Workspace: `target/release-demo/commit-boundary-20260902-123144-595`
- Run: `47da6a06-a3f3-4b66-8425-838efd8a7809`
- Base commit: `ca15aa66dbf2b25fe00aeeaac9b6ebdb8f28a480`
- Package: `30b29840dc68225b7a521ce0b78ca144d7a65c8555bc71af39f781f784e086a9`
- After review, the operator changed affected path `src/risk-policy.mjs`.
- Confirmed Apply returned “Review is stale”; README and the test file were not
  applied, and only the operator edit remained.
- Baseline tests still passed 2/2.
- History showed Completed / Apply failed and no success receipt.
- Captures: `target/release-demo/final-native-stale-refusal.png` and
  `final-native-stale-history.png`.

## Verification commands and results

### Repository and Rust — VERIFIED

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p pytxo-cli --release
cargo build -p pytxo-mcp --release
cargo run -p pytxo-cli -- status --json
cargo run -p pytxo-cli -- --version
git diff --check
```

All commands exited 0 on the final code candidate. The workspace test includes
all unit, integration, process, and doc-test targets: 69 core, 50 runner, 32
change-set, 9 Apply, 4 dispatch-lifecycle, 3 dependency-outcome, 3 Stop, 22
Desktop-native, 22 Link, and 6 cloud-sandbox tests among the passing targets.
The rebuilt CLI reports `pytxo 1.2.2`; status returned valid JSON with the
persisted domain isolation mechanism.

The v1.2.2 corrective tree was verified on 2026-09-04 with formatting, the
complete workspace test suite, warnings-denied clippy, optimized CLI and MCP
builds, and the expanded release-version and inventory contracts. All exited 0.

The repaired 69-test core suite additionally passed 10 consecutive parallel
runs. The exact Windows descendant Stop test and PTY environment isolation test
passed after rejecting a regressing `portable-pty 0.9` upgrade.

### Desktop — VERIFIED (patch scope)

```powershell
cd apps/desktop
npm run check
npm run build:native
cargo test -p pytxo-desktop
```

- On v1.2.2, Svelte reported 0 errors/warnings, CSS lint passed, the frontend
  transformed 6,366 modules, the optimized Windows native build passed, and all
  22 Desktop-native tests passed in the workspace run.
- Product UI source is unchanged from v1.2.1. Its production-preview browser
  suite (104/104), development-only legacy-deck suite (1/1), Storybook
  interaction/accessibility suite (41/41), and packaged runtime proofs remain
  the relevant behavior evidence.

### Website and public docs — VERIFIED

```powershell
cd apps/web
pnpm install --frozen-lockfile
pnpm run lint
pnpm run verify:product-assets
pnpm run check:links
pnpm run build
pnpm run e2e -- e2e/marketing.spec.ts --workers=1 --reporter=line
pnpm audit --prod --audit-level=high
```

On v1.2.2, lint, 27 product-asset references, 153 internal links, and the
58-page production build passed after updating public version and Windows-only
Desktop copy. The unchanged product pages retain the v1.2.1 17-test browser and
zero-production-vulnerability evidence.

### Demo — VERIFIED SILENT / BLOCKED NARRATED

The v1.2.2 typecheck, composition list, asset validator, audio parser tests, and
publishing validator passed. The unchanged v1.2.1 demo clean install, production
audit, still/poster generation, silent render, silent validation, and transition
sheet remain valid. The master is
52.000 seconds, 1920×1080, 30 fps, BT.709, H.264. The transition sheet was
visually reviewed and uses one coherent current Work/Run Review shell.

Narrated publication is **BLOCKED** until approved/licensed audio is supplied
and `validate:narrated` passes. The release may claim only the silent master.

### npm package and staged install — PARTIALLY VERIFIED

```powershell
cd packages/pytxo
npm test
npm pack
```

- v1.2.2 installer/platform/checksum/atomic tests: 4/4 passed against the rebuilt
  release CLI, including embedded-version and checksum refusal.
- v1.2.2 dry-pack: exactly six intended files; zero runtime dependencies.
- The exact final tarball installed into fresh prefix
  `target/final-package-install-20260902-155602-514` through the production
  postinstall path.
- Downloaded CLI SHA-256:
  `aab81760200de903cb06c35f3a5e324bdfbf241ee1e5fa6a882c131b09517bda`.
- The original exact-package rehearsal reported `pytxo 1.2.0`; after discovering
  that tag was already public, the retargeted v1.2.1 wrapper tests, dry-pack,
  source parity, optimized binary build, and live `pytxo 1.2.1` check passed.
- A clean install from the public `pytxo@1.2.2` package remains required before
  this section can become fully verified.

### Security and artifact audit — VERIFIED WITH ACCEPTED WARNINGS

- `cargo audit`: 0 unignored vulnerabilities; 17 unmaintained informational
  warnings, 1 Linux-only GLib soundness warning, and 1 unenabled yanked `spin`
  dependency warning.
- Ratatui/LRU and optional FUSE soundness advisories discovered during the audit
  were removed by dependency upgrades.
- Desktop, web, and demo production dependency audits report zero known
  vulnerabilities.
- The npm wrapper has no runtime dependencies and therefore no lockfile audit
  surface; its exact tarball and downloaded binary were inspected instead.
- Tracked high-confidence secret scan matched only synthetic sanitizer/upload
  fixtures in test-bearing source files. No tracked absolute developer path was
  found. User-owned untracked captures/logs/notes are not part of a clean release
  checkout and were preserved.
- Version parity verifies 15 first-party Cargo lock entries plus Rust, Desktop
  and demo package/lock metadata, Tauri, npm, web downloads, the PowerShell
  installer, README, public changelog, and release notes at v1.2.2.

## Demo readiness

`DEMO.md` is the canonical 2–4 minute sequence. `GROK_DEMO_BRIEF.md` adds a
75-second launch treatment, exact narration, generated bridge prompts, truthful
capture rules, five gallery frames, social crops, and failure fallbacks. The
canonical walkthrough includes the first-20-second
product promise, exact commands and clicks, expected evidence, a controlled
stale-path insert, fallback behavior, narration, recording checklist, suggested
shots, and release/social descriptions. `prepare.ps1` creates a deterministic
fixture plus isolated `PYTXO_HOME` and WebView2 user-data stores so personal
workspaces and recents cannot leak into a take.

## Deployment and release readiness

- The canonical workflow gates publication on full CI, version parity,
  protected-main ancestry, exact five-platform CLI artifacts, exact one-platform
  Windows Desktop artifacts, checksums, and required mirror credentials.
- The bypass Desktop publisher was removed; every referenced third-party Action
  is pinned to an immutable revision.
- npm postinstall and direct installers fail hard on missing asset, checksum, or
  embedded-version mismatch and replace atomically.
- Updater metadata requires exactly the declared Windows platform. The repository
  has a Tauri updater-signing secret; Windows Authenticode credentials are absent.
- v1.2.1 subsequently passed protected CI (12/12 jobs) and the complete release
  workflow. v1.2.2 must repeat those hosted gates through the corrected workflow
  before it replaces the repaired-but-mutable v1.2.1 artifact history.

## Merchant-of-record architecture

- Dodo is the intended Merchant of Record/legal seller; MBCZ is the verified
  business-account owner and payout beneficiary; Pytxo is the customer-facing
  brand. The legal/KYC layer remains outside this repository.
- Proposed `ADR-0040` and `docs/01-projects/dodo-mor-integration.md` define a
  provider-neutral Dodo-to-Link adapter behind a raw Pytxo-domain proxy, with
  replay, ordering, grant aggregation, allowlist, and migration requirements.
- This is **IMPLEMENTED as documentation** and **UNVERIFIED as runtime**. v1.2.2
  does not claim a Dodo checkout or webhook cutover; the existing Paddle adapter
  remains authoritative until test-mode and dual-run evidence exist.

## Known limitations and accepted risks

- Reviewed Apply covers Orbit/Galaxy, one execution domain, one repository root,
  and whole prepared paths—not cross-root transactions or partial-file approval.
- The Apply journal supports process-crash reconciliation, not claimed power-loss
  ACID or cross-filesystem atomicity.
- Isolation is platform-dependent; receipts expose enforced, advisory,
  unavailable, or bypassed mechanisms. Supernova is intentionally host-direct.
- Desktop v1.2 is Windows-first. The workflow rejects unexpected macOS/Linux
  Desktop artifacts; older installers must not be relabeled as v1.2.
- Regex sanitization is defense in depth, not an authorization boundary or proof
  that arbitrary output is secret-free.
- Link, cloud, and proxy production health endpoints passed after the v1.2.1
  deployment. A live production PostgreSQL migration and authenticated sandbox
  sync-to-exec transaction remain unverified and outside the default local v1
  claim.
- `portable-pty 0.8.1` retains an unmaintained `serial` dependency. Its maintained
  0.9 upgrade caused a reproducible PTY lifecycle hang and was rejected. There is
  no RustSec vulnerability advisory for `serial`.
- Tauri's Linux-only GTK3 graph carries an old GLib iterator soundness advisory
  and unmaintained GTK3 warnings; Linux Desktop is not shipped in v1.2.
- Windows notification support pins `quick-xml 0.37.5`, with two ignored XML DoS
  advisories. The parser receives only Pytxo-generated notification XML, not
  untrusted remote XML. The ignored RSA timing advisory is absent from the
  Windows graph and is not used for Pytxo key handling.
- A narrated master is unavailable until approved/licensed audio is provided.

## Final recommendation

**NOT READY.**

The local Windows-first product claim, main success path, critical stale-path
failure, UI behavior, tests, builds, dependency posture, and silent demo are
supported by current evidence. The v1.2.2 corrective source also passes its full
local patch-scope gate. It remains not ready until protected CI passes, the fresh
tag publishes through the corrected workflow, and independent downloads prove
the exact private/public inventories, checksums, embedded versions, updater
signature, and npm install. If those checks pass, the appropriate final rating
is **READY WITH KNOWN RISKS**, limited by absent Authenticode/Git signing,
licensed narration, and the documented runtime boundaries above.
