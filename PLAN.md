# Pytxo v1.1.1: reviewed mission control polish

Status: implementation complete; final release verification and publication pending.

## Phase 1: reviewed bytes and durable Apply

- [x] Persist the plan, base revision, permission profile, enforcement receipt,
  review state, and Apply history as one run contract.
- [x] Prepare an immutable package immediately after each successful Orbit or
  Galaxy run. Blobs preserve exact bytes for additions and modifications; the
  manifest records deletions, untracked files, binaries, digests, sizes, and
  ownership.
- [x] Keep dependent-wave workspaces until the package is durably staged.
- [x] Apply only the prepared package. Validate affected-path preimages while
  allowing unrelated checkout changes.
- [x] Reject unsafe paths, special files, symlinks, traversal, protected Pytxo
  paths, and divergent ownership.
- [x] Journal every Apply attempt under
  `.pytxo/data/apply/<run-id>/<attempt-id>/` and reconcile interrupted attempts
  before another run or Apply in that execution domain.
- [x] Keep a package retryable after a confirmed rollback; mark affected-path
  drift stale; block unprovable recovery as `recovery_required`.
- [x] Support explicit refresh, retry, recovery reconciliation, and discard.
- [x] Keep DeepSpace non-flushable and Supernova host-direct.

## Phase 2: Flow as the Desktop mission home

- [x] Put Compose, Active, History, and contextual Run Review inside Flow.
- [x] Redirect legacy Runs and Run Review deep links into Flow.
- [x] Show the immutable package, exact file changes, base revision, package
  digest, ownership DAG, permission profile, enforcement receipt, and attempt
  history in Run Review.
- [x] Guard Apply against double submission and explain disabled actions.
- [x] Combine immediate Tauri mutation events with cursor-based catch-up and an
  infrequent integrity refresh.
- [x] Keep structural Focus contextual and load the legacy Deck only behind its
  development flags.
- [x] Verify keyboard behavior, focus, reduced motion, status announcements,
  and 1600x1000, 1280x800, and 960x640 layouts.

## Phase 3: product evidence and release material

- [x] Recut the Remotion demo to 52 seconds around Flow, Run Review, Apply,
  Operations, and the final product line.
- [x] Remove burned subtitles and keep an external transcript and provisional
  SRT for accessible publishing. Retiming remains required against the approved
  continuous narration.
- [x] Render and validate the silent 1920x1080 H.264 master and poster.
- [x] Update the homepage, public docs, architecture notes, deterministic
  product captures, and v1.1 release notes.
- Publication remains blocked on approved audio and final repository gates;
  the executable blockers are tracked below.

## Deferred by design

- Cross-root transactions and partial-file acceptance need separate contracts.
- v1.1 does not claim power-loss ACID behavior or cross-filesystem durability.
- Kernel-grade Orbit and Galaxy filesystem/network boundaries remain platform
  work; receipts continue to label advisory controls as advisory.

## Release workflow

Use the maintained [release workflow](docs/07-guides/release-workflow.md) after
every command below passes on the final tree. Run commands from the repository
root unless a block changes directory.

### Rust and repository contract

```powershell
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo build -p pytxo-cli --release
cargo build -p pytxo-mcp --release
cargo run -p pytxo-cli -- status --json
```

### Pytxo Desktop

```powershell
Push-Location apps/desktop
npm ci
npm run check
$env:PLAYWRIGHT_CHANNEL='chromium'
npm run e2e
npm run storybook:build
npm run storybook:test:ci
npm run build:native
Pop-Location
cargo test -p pytxo-desktop
```

### Web and public docs

```powershell
Push-Location apps/web
pnpm install --frozen-lockfile
pnpm run lint
pnpm run verify:product-assets
pnpm run check:links
pnpm run build
pnpm exec playwright install chromium
pnpm run e2e -- e2e/marketing.spec.ts --workers=1 --reporter=line
pnpm audit --prod --audit-level=high
Pop-Location
```

### Product demo

```powershell
ffmpeg -version
ffprobe -version
Push-Location apps/demo-video
npm ci
npm run typecheck
npm run compositions
npm run test:audio-qa
npm run validate:publishing
npm run stills
npm run poster
npm run render:silent
npm run validate:silent
npm run transition-sheet
Pop-Location
```

After the approved audio assets are present, run the narrated gates separately:

```powershell
Push-Location apps/demo-video
npm run voiceover
npm run render:narrated
npm run validate:narrated
Pop-Location
```

### Final repository and publication checks

```powershell
git diff --check
git status --short
cargo run -p pytxo-cli -- --version
```

Confirm `distribution/release-notes/v1.1.1.md`, `RELEASE.md`, package versions,
installer metadata, and website download metadata all name v1.1.1. Then follow
the linked workflow to create the tag, installers, checksums, npm package,
public mirror, website deployment, and post-release download checks.

## Release acceptance criteria

- All commands above exit zero on the final tree and the production web audit
  reports zero high or critical vulnerabilities.
- Reviewed Apply tests cover exact stored bytes, affected-path drift, retry,
  stale and recovery states, one execution domain, and one repository root.
- Desktop tests cover Flow, Run Review, Apply guards, recovery, keyboard and
  reduced-motion behavior, and 1600x1000, 1280x800, and 960x640 layouts.
- Product assets match `apps/desktop/captures/desktop-2/` byte for byte; the
  verifier must fail when that canonical source directory is absent.
- The silent demo master passes publishing and media validation with no burned
  subtitles. The external SRT remains provisional until retimed.
- Version metadata, release notes, installers, checksums, npm, public mirror,
  download page, and documentation all resolve to the same v1.1.1 artifacts.
- No tag, installer, website, or demo is published before the final repository
  gates and both blockers below are cleared.

## Publication blockers

- [ ] Generate and approve the continuous narration, license the recorded
  music and three interface cues, retime the provisional SRT, and pass
  `npm run validate:narrated`.
- [ ] Publish installers, tag, npm package, public mirror, website, and demo
  only after the repository-wide release checklist passes on the final tree.
