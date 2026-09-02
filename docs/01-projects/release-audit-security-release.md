---
title: Public v1 security and release audit
slug: release-audit-security-release
status: active
tags: [project, release, audit, security, distribution]
audience: [human, agent]
layer: orchestration
created: 2026-08-31
updated: 2026-08-31
related: [[vision]], [[commit-layer]], [[permission-profile-engine]], [[regex-sanitization]], [[ADR-0038-epistemic-state-contract]]
---

# Public v1 security and release audit

## Verdict

**Do not treat the current build or distribution process as a credible public v1.** Three P0 security boundaries can be bypassed or omitted, and the current public `v1.2.0` distribution proves that the release workflow can publish an npm package whose advertised platforms do not exist. The reviewed Apply implementation and its receipts contain good, honest controls, but those controls do not compensate for a repository-controlled Supernova override, raw cloud uploads, or an unsigned public entitlement webhook.

This audit inspected commit `f31a75a` on `release/1.2.0`. The worktree was concurrently modified during the audit; line references below describe the state present when this note was finalized, and uncommitted changes are **not** counted as shipped fixes. Severity means: P0 = immediate trust/credential/privilege boundary failure; P1 = public-release blocker; P2 = important gap that must be disclosed or fixed; P3 = hardening. “Verified” means exercised or checked during this audit; “implemented” means supported by code inspection; “unverified” means deployment/runtime evidence was unavailable.

## Highest-priority findings

### P0.1 — Folder trust does not cap repository-controlled per-agent profiles

**Implemented and statically verified; exploit integration test absent.** `load_config` reads the repository's `pytxo.toml` and replaces only the global profile with the out-of-band trusted-domain choice (`crates/pytxo-orchestrate/src/lib.rs:2066-2085`). `agent_profile_map` still accepts each `[[agent]].permission_profile` from that file (`crates/pytxo-core/src/config.rs:249-266`), and the entitlement layer caps it only against an optional organization ceiling (`crates/pytxo-orchestrate/src/entitlements.rs:120-135`). The runner selects that per-agent value (`crates/pytxo-runner/src/run.rs:547-558`); Supernova then deliberately skips isolation and runs at the repository root (`crates/pytxo-runner/src/run.rs:574-586`).

An untrusted repository can therefore request `permission_profile = "supernova"` for its agent, the operator can trust the folder at Orbit, and that agent still receives host-direct writes. The stored per-agent receipt will truthfully say Supernova, but the trust prompt has already been defeated. The stdin and MCP gates compound this: both consult only the global profile, not the live agent's effective profile (`crates/pytxo-orchestrate/src/lib.rs:291-321`, `392-420`).

**Fix:** make the trusted-domain profile an absolute ceiling for global, per-agent, and project-root profiles. Reject rather than silently elevate any repository request above it; require a distinct out-of-band confirmation to raise the ceiling. Resolve live-agent profile from the run envelope for stdin/MCP gates.

**Verify:** add an integration test that trusts a fixture repository at Orbit, gives its agent a Supernova override, dispatches it, and proves the effective receipt remains Orbit and `workspace.cwd != repo_root`. Add Galaxy-agent/global-Orbit stdin and MCP tests that require HITL.

### P0.2 — Cloud execution sends repository content without Sovereign Shield sanitization

**Implemented and statically verified; production retention and transport unverified.** When cloud execution starts, the runner collects and sends repository files directly (`crates/pytxo-runner/src/run.rs:703-721`). `collect_sync_paths` recursively reads every non-binary text file up to 512 KiB and puts its raw content on the wire (`crates/pytxo-core/src/cloud/delta.rs:93-151`); changed overlay bytes are also converted directly to strings (`crates/pytxo-core/src/cloud/delta.rs:18-70`). Default exclusions cover only `node_modules`, `.git`, `target`, `dist`, and `build`, not `.env`, `.pytxo`, private keys, cloud credentials, or provider config (`crates/pytxo-core/src/config.rs:24-31`). The context cache similarly uploads generated/raw fallback content without a sanitizer (`crates/pytxo-runner/src/context.rs:105-138`). Max/Ultra entitlement automatically enables cloud and changes PTY execution to Cloud (`crates/pytxo-orchestrate/src/lib.rs:1294-1301`).

This contradicts the security note that files are locally sanitized before being sent to cloud and that matching is a multi-threaded scanner (`docs/02-areas/security/regex-sanitization.md:15-25`). The actual sanitizer is a sequential pass over five regexes (`crates/pytxo-sanitize/src/lib.rs:4-34`) and is not on these upload paths. Cloud is disabled in the base config (`crates/pytxo-core/src/cloud/config.rs:29-37`), which limits exposure for local Core, but enabling a paid cloud tier exposes ordinary repositories to this path.

**Fix:** use an explicit, displayed sync manifest; deny secret-bearing paths by default; honor git ignore plus a dedicated cloud denylist; scan content and fail closed with a receipt when a likely secret is found. Do not silently redact executable source in ways that change semantics. Require informed consent before the first upload and document server retention/deletion.

**Verify:** cloud integration fixtures must contain `.env`, `.pytxo`, PEM, GitHub/AWS/Anthropic/OpenAI tokens, database URLs, and nested credential files; none may reach the mock server. Test cache puts and overlay deltas separately, with `sanitize=true` and `false` explicitly defined.

### P0.3 — Paddle entitlement changes are accepted without a signature when the secret is absent

**Implemented and statically verified; public deployment configuration unverified.** `/v1/webhooks/paddle` is always routed (`services/pytxo-link/src/main.rs:532-553`). Signature checking occurs only if `PADDLE_WEBHOOK_SECRET` exists and is non-empty; otherwise the body is processed (`services/pytxo-link/src/main.rs:380-405`). The body controls `user_id` and tier, including Ultra, and every non-cancellation event updates the entitlement (`services/pytxo-link/src/paddle.rs:64-96`). Deployment documentation calls the secret optional (`distribution/railway/README.md:375-384`; `services/pytxo-link/railway.json:28-30`). Even when enabled, verification accepts any signed timestamp and has no event-id replay protection (`services/pytxo-link/src/paddle.rs:23-51`).

Anyone who can reach a Link instance with the documented optional-secret configuration can upgrade an arbitrary user. This is a public-release blocker even if the intended production route currently goes through the web application.

**Fix:** fail startup, or make the route unavailable, unless a webhook secret is configured. Always verify the signature, enforce a short timestamp tolerance, allowlist event types, bind customer/subscription identity to server-side records, and persist Paddle event IDs transactionally for idempotency.

**Verify:** HTTP tests must reject missing-secret startup, unsigned/incorrect/stale signatures, invented event types, tier changes from untrusted `custom_data`, and replayed event IDs. `cargo test -p pytxo-link -- --list` currently shows no Paddle tests.

## P1 public-release blockers

### P1.1 — The release workflow publishes partial matrices and a cross-platform npm wrapper

**Verified in workflow and current public state.** The CLI release runs after the matrix even when matrix legs fail and requires only one `dist/pytxo-*` file (`.github/workflows/release.yml:107-129`). npm depends only on that partial release (`.github/workflows/release.yml:444-450`). The wrapper advertises Linux x64/arm64, macOS x64/arm64, and Windows x64/arm64 (`packages/pytxo/lib/platform.js:5-18`), although the workflow never builds Windows ARM (`.github/workflows/release.yml:43-69`). Its postinstall catches download failure and exits successfully (`packages/pytxo/postinstall.js:40-62`).

On 2026-08-31, npm `latest` was `1.2.0`, but the public `v1.2.0` release contained only `pytxo-windows-x64.exe`, two Windows Desktop installers, and `SHA256SUMS.txt`. Requests for Linux x64 and Windows ARM returned 404. Thus a Linux/macOS `npm i -g pytxo@1.2.0` can report success while leaving no usable binary. This conflicts with the five-platform install table and npm instructions (`apps/web/content/docs/getting-started/install.mdx:11-51`) and with the after-release requirement to verify all five (`docs/07-guides/release-workflow.md:76-80`).

**Fix:** require the exact declared asset set before creating the release or publishing npm. Prefer platform-specific npm packages with a thin selector, so one failed platform cannot poison every install. Remove Windows ARM claims until it is built. Publish an availability manifest consumed by installers, npm, and the website.

**Verify:** in fresh Windows x64, Windows ARM64, macOS x64/arm64, and Linux x64/arm64 environments, install the exact npm version, assert postinstall fails if its asset is absent/tampered, and run `pytxo --version` plus `pytxo doctor`.

### P1.2 — Update and installer integrity are not end-to-end

**Verified.** The configured updater endpoint is `releases/latest/download/latest.json` (`apps/desktop/src-tauri/tauri.conf.json:63-67`), but it returned 404 for the latest public release. The release guide incorrectly says omitting a new unsigned manifest leaves the channel on the last signed release (`docs/07-guides/release-workflow.md:72-74`); GitHub's `latest` URL resolves to the newest release, where the asset is absent. The main desktop mirror can run without requiring the desktop matrix to succeed (`.github/workflows/release.yml:371-377`) and can add partial manual installers.

All three CLI install paths ignore the published checksum: shell downloads to a temporary file but runs doctor best-effort (`distribution/pytxo-releases/install.sh:44-62`); PowerShell overwrites the final executable directly (`distribution/pytxo-releases/install.ps1:14-28`); npm opens the final vendor file before validating status, leaves partial/zero-byte files on failure, and later skips any existing file (`packages/pytxo/postinstall.js:18-62`). Desktop also writes directly to the final path, checks only that a file exists, and treats doctor failure as a note (`apps/desktop/src-tauri/src/ipc_install.rs:203-252`, `286-300`). The workflow generates CLI checksums before Desktop is added, while desktop mirror jobs do not regenerate them (`.github/workflows/release.yml:139-143`, `371-440`; `.github/workflows/desktop-release.yml:182-198`).

**Fix:** sign a release manifest that binds version, platform, filename, size, digest, and source commit; verify it before activation; download to a sibling temporary file and atomically rename only after digest, signature, embedded version, and doctor pass. Make npm postinstall fail hard. Either publish a signed updater manifest on every latest release or use a stable signed-channel URL independent of manual releases.

**Verify:** corrupt and truncate every asset, interrupt each installer mid-download, rerun it, and prove the old binary remains intact. Check that every public asset is covered by the signed manifest and that updater checks return a valid current or explicit no-update response, never 404.

### P1.3 — Release publication is not gated on CI or a tested commit

**Implemented; branch protection unverified.** CI runs on `main` pushes and pull requests (`.github/workflows/ci.yml:3-7`), but the tag/dispatch release workflow has no dependency on CI and performs builds/version checks without tests, clippy, audits, web checks, secret scanning, or fresh-install tests (`.github/workflows/release.yml:42-171`). A tag can therefore publish any commit that compiles. All Actions dependencies use mutable tags such as `@v4`, `@stable`, `@v2`, and `tauri-action@v0`, and Rust itself is `stable`, reducing provenance and reproducibility.

**Fix:** make release consume artifacts from a required reusable gate for the exact commit; require the tag commit to be reachable from protected `main`; pin Actions by commit SHA and Rust by version; use `--locked`; add SLSA provenance/SBOM and a real secret scanner.

**Verify:** attempt release from a failing test commit and a commit not on `main`; both must stop before any private/public release or npm mutation. Verify artifact attestations against the tag SHA.

### P1.4 — Hosted service authentication defaults fail open

**Implemented; production environment unverified.** Link defaults `LINK_REQUIRE_AUTH` to false (`services/pytxo-link/src/main.rs:451-456`); its authorization helper then accepts an anonymous subject (`services/pytxo-link/src/auth.rs:39-42`). Cloud sandbox similarly defaults authentication according to whether a key happens to exist (`services/pytxo-cloud-sandbox/src/main.rs:714-719`). Railway injects a public bind through `PORT`, so an omitted variable can expose operational APIs. Documentation says to set auth, but secure production behavior should not depend on remembering it.

**Fix:** fail startup on public binds/`PORT` unless authentication and required keys are configured; make insecure local mode an explicit loopback-only flag. Add deployment preflight and health metadata that reports auth mode without exposing keys.

**Verify:** start each service with production bind variables but missing auth/key and assert non-zero startup; test anonymous access to every non-health route.

### P1.5 — The web-to-Desktop account handoff both leaks a bearer and cannot succeed

**Implemented and statically verified; production Clerk flow unverified.** The account page obtains a Clerk token and places it in `pytxo-deck://auth?token=...` (`apps/web/src/components/account-auth.tsx:67-106`). Desktop still opens that page (`apps/desktop/src-tauri/src/ipc_auth.rs:57-62`) but intentionally rejects every legacy bearer callback and demands a one-time authorization code with PKCE (`apps/desktop/src-tauri/src/ipc_auth.rs:217-225`). The current Desktop Settings UI says the connection is paused, but the web return path remains active. Custom-scheme URLs can be observed by browser/OS handlers and are not an appropriate bearer transport.

**Fix:** disable the web return link until a server-issued, single-use, short-lived code with state and PKCE is implemented. Never put a reusable Clerk session bearer in a URI.

**Verify:** a full browser-to-installed-Desktop test must prove state/PKCE validation, one-time redemption, replay rejection, cancellation, wrong-client rejection, and no bearer in URLs/logs/history.

## P2 findings

- **Sanitizer coverage and error paths:** only OpenAI-style `sk-`, bearer, generic `api_key`/`token`, and home paths are covered (`crates/pytxo-sanitize/src/lib.rs:4-34`). GitHub, AWS, Anthropic, PEM, cookies, passwords, and credential URLs are absent. MCP sanitizes success text but emits error strings raw (`crates/pytxo-mcp/src/main.rs:157-177`), and MCP audit payloads are stored raw regardless of `cfg.sanitize` (`crates/pytxo-orchestrate/src/lib.rs:380-388`). Expand structured secret detection, sanitize every outbound/error/audit path, and add representative fixtures.
- **Secondary Desktop workflow cannot find workspace artifacts:** Cargo metadata reports `target_directory = C:\pytxo\target`, while `.github/workflows/desktop-release.yml:123-129` uploads from `apps/desktop/src-tauri/target/...`. The recommended main release uses the correct root path. Fix the dedicated workflow and test its artifact contract.
- **npm license omission:** the root MIT license exists, but the package allowlist omits it (`packages/pytxo/package.json:27-31`). `npm pack --dry-run --json` confirmed the published tarball has only five files and no license. Add `LICENSE`, third-party notices, SBOM/license policy, and verify packed contents.
- **Dependency and workflow supply chain:** `cargo audit` exited successfully but reported 23 allowed warnings, including unmaintained GTK-related crates and unsound `fuser`, `glib`, and `lru`; three advisories are explicitly ignored (`.cargo/audit.toml:1-13`). Triage with reachability/platform notes and an owner/expiry date. Pin external Actions and publish provenance.
- **Security and release documentation drift:** `SECURITY.md` supports only `0.1.x`, calls Desktop “Reality Deck,” omits web/cloud/link/proxy scope, and points public reporters to a private repository (`SECURITY.md:3-22`). The root README says Install v0.3.0 and Status v0.1.0 (`README.md:23-30`, `72-74`); `CHECKPOINT.md` is v1.0.0 (`CHECKPOINT.md:1-5`); the public changelog lists v1.1.1 although the public release sequence jumps from v1.1.0 to v1.2.0 (`apps/web/content/docs/reference/changelog.mdx:10-14`). Replace these with one generated release identity and a public security contact.
- **Public provenance is incomplete:** the local/source history has no `v1.2.0` tag (`git describe` was `v1.1.0-17-gf31a75a`), and the public distribution release targets the releases repository's `main`, not a source commit. The binary embedded version/source commit was not verified. Bind public artifacts and npm metadata to a source tag/commit attestation.

## P3 hardening and hygiene

- `apps/web/vercel.json:93-99` sets frame and referrer policy but no CSP, `X-Content-Type-Options`, or `Permissions-Policy`; the root layout loads third-party Fontshare CSS (`apps/web/src/app/layout.tsx:47-53`). Add a tested CSP and baseline headers.
- High-signal tracked-file scanning found no GitHub PAT, AWS access key, or private-key marker; OpenAI-looking matches were test fixtures only. This was not a full history/entropy scan, and no CI secret-scanner job exists.
- The worktree contains untracked captures, `.verify` output, `desktop-e2e.log`, and `docs/superpowers/`. They were not modified by this audit. Add targeted ignores or release-cleanliness checks so they cannot enter a source archive accidentally.

## Controls verified during this audit

- `node tooling/scripts/verify-release-version.mjs 1.2.0` passed across Rust, npm, Desktop, Tauri, and demo manifests; its two unit tests passed.
- The local npm dry-run integrity and shasum exactly matched published `pytxo@1.2.0`. This verifies npm package freshness, not its downloaded executable.
- Tag-trigger npm normalization is **not** the suspected bug: the workflow imports `VERSION` as an exported step environment variable, strips `v`, and child Node inherits the normalized value (`.github/workflows/release.yml:455-469`); npm also accepts `pytxo@v1.2.0`. Add a workflow-level tag fixture, but do not report this path as broken.
- The public install scripts matched the checked-in scripts after newline normalization. Public `SHA256SUMS.txt` matched GitHub's asset digests for all four actually published files. Consumers still do not verify it.
- `cargo test -p pytxo-sanitize` passed 2 tests; orchestrate entitlement tests passed; runner enforcement receipts passed 2 tests and network policy passed 4. These validate limited controls, not the P0 gaps above. `cargo test -p pytxo-link paddle` selected zero tests.
- Enforcement receipts are honest about Orbit/Galaxy host-filesystem and network controls being advisory and DeepSpace fails closed when a recognized socket isolation mechanism is absent (`crates/pytxo-runner/src/enforcement.rs:21-140`).

## Required release gate

After fixes, run from a clean checkout and retain logs/artifact digests:

```powershell
node tooling/scripts/verify-release-version.mjs 1.2.0
node --test tooling/scripts/verify-release-version.test.mjs
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo audit
Push-Location apps/desktop; npm ci; npm run check; npm run build:native; Pop-Location
Push-Location apps/web; npm ci; npm run lint; npm run build; npm audit --omit=dev --audit-level=high; Pop-Location
Push-Location packages/pytxo; npm pack --dry-run --json; Pop-Location
```

Then require an automated release-manifest check that compares the exact expected platform set, verifies every checksum/signature and embedded version, confirms updater `latest.json`, and performs fresh npm/script/Desktop installs on each claimed platform. Run dedicated negative security tests for trust-profile escalation, cloud secret sync, unsigned/replayed Paddle webhooks, anonymous public services, and account-code replay before promoting npm `latest` or GitHub “Latest.”
