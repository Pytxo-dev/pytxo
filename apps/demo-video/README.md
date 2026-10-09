# Pytxo product demo

## Live film v8 - October 10

Current direction: **`PytxoLive`** cut from one native journey take of the
released v1.2.4 Desktop (live take 2, `D:/pytxo-native-acceptance/film-124-live2`):
Claude Code splits a one-paragraph request into 7 owned tasks; Claude Code,
Codex and Cursor Agent run them; Review, stale refusal, refresh and Apply are
the real recorded UI. The pointer is redrawn from the take's telemetry. Waits
are shortened and each shortened shot shows "sped up N×"; the fleet beat is a time-lapse of the
whole 11-minute run. The result-app segment is left out (`--no-result`): the
applied app has integration bugs that the agents' tests did not catch.

```powershell
# take (drives the real pointer; leave the machine alone ~20 min)
tooling/acceptance/run-local.ps1 -Exe <released exe> -Evidence <dir> -Python <venv python> `
  -Film -Live -Split -Lead "Claude Code" -Team "OpenAI Codex,Cursor Agent"
node scripts/live-cut.mjs <dir>/journey --no-result
./scripts/render-live.ps1 -Out out/pytxo-live-v8.mp4 -Fresh
```

Audio (`public/live/audio/`: ElevenLabs voiceover, CC0 music and effects) is
ignored by git and was copied from the v7 working set.

## Motion recut with CC0 audio - October 3

Current direction: **`PytxoMotionFilm`**, a 50-second connected motion edit
with real recorded music, sparse editorial sound effects and native evidence.
See [MOTION-CUT.md](MOTION-CUT.md) for the edit, source provenance, audio
rights and cloud-only rendering/QA. This supersedes the earlier silent-only
direction below. The silent film is preserved as a separate historical cut.

The private GitHub Actions workflow renders `out/pytxo-motion.mp4` and retains
the master, review frames and audio measurements. No public release or
Windows native acceptance is implied by a successful film render.

## Silent beta film - October 3

`PytxoBetaFilm` is a 56-second, 1920x1080, 60 fps motion-graphics edit: large
Geist type, neutral white space, restrained motion and recorded native receipts.
The final export is `out/pytxo-beta-silent.mp4`. No audio track is included, per
Matt's October 3 direction. The previous compositions remain available.

```powershell
cd apps/demo-video
npm run typecheck
npm run validate:beta
npm run studio -- --port=3100
npm run render:beta
npm run validate:beta:master
```

The source is `src/beta/`; editable scene timings are in `PytxoBetaFilm.tsx`.
Scenes: introduction (0-5 s), request (5-11), ordered fleet (11-20), combined
package (20-28), stale refusal (28-35), recorded Apply (35-42), resulting task
board (42-49), closing (49-56). Long holds are deliberate reading time.

Evidence: the same October 2 run described below, **not a new run on the final
candidate**. The film combines ledger-driven diagrams with cropped native
stills; it is not continuous footage or a simulated live interaction. No
pointer, elapsed-time performance claim, or six-vendor success is invented.
Both no-change workers remain visible. `validate:beta` checks the depicted run,
worker outcomes, package ownership, counts and native-image SHA-256 values.
`validate:beta:master` checks the exported stream, duration, silence and decode.

The final export played to completion at normal speed in Chrome, with no media
error; all scenes were also inspected as rendered frames. Final-build continuous
native capture, independent privacy review and publication approval remain
separate release gates. A 60 fps edit does not make its stills 60 fps product capture.

## Fleet film — October 2 draft

`PytxoFleetFilm` (52 s, 1920×1080, 30 fps) is the approved fleet storyboard:
terminal wall → wave graph with the shared-file ordering → the recorded fleet
board → one exact package with its checks and digest → stale refusal → Apply →
the resulting app → end card. Every count, vendor, path, digest and worker line
comes from `fleet-props.json`, which `scripts/fleet-ledger.mjs` exports from one
recorded native run of `docs/demo/fleet`. Worker lines are allow-listed (commands,
check results, added code, lines about owned files) because workers read the
operator's own agent configuration; host paths and secrets never pass.

```powershell
cd apps/demo-video
npm run fleet:ledger -- --evidence D:/pytxo-native-acceptance/<root> `
  --repo <fixture repo> --stale-digest <refused digest> --stills <dir of key PNGs>
npm run typecheck
npm run render:fleet
```

The current props and `public/fleet/*.png` come from the October 2 run on MSI
`0D20D3B8…` (`D:/pytxo-native-acceptance/fleet-20261002b`), where OpenCode and
Antigravity prepared no files; the film shows them as "no changes". The draft
uses native stills, so its disclosure reads "native stills, continuous capture
pending". Before publication: repeat the run on the final MSI with OpenCode's
provider stored, replace the stills with continuous native capture, and cut to
the licensed track. The Antigravity mark is drawn from
`public/fleet/logos/antigravity.png` (Matt approved public use on 2026-10-02;
see `apps/desktop/public/ade/PROVENANCE.md`).

## Current R6 cockpit proof and recording boundary — September 21

`PytxoR6CockpitProof` is a 12-second review composition bound to the exact local
1.2.2 R6 candidate. It uses two hash-locked native frames: the clean first-use
screen and the 24-worker, eight-wave canvas at 39% Fit. The dense frame retains the
minimap and the explicit `Review changes` action while proving that the redundant
bottom commit rail is absent.

The proof uses the Chroma Aperture palette, a 12 fps ambient ASCII glyph, restrained
camera drift and sparse product copy. It does not simulate a pointer, worker motion,
elapsed time or Apply. The disclosure `R6 cockpit proof · exact native stills ·
continuous capture pending` remains visible throughout.

```powershell
cd apps/demo-video
npm run validate:r6-cockpit
npm run typecheck
npm run render:r6-cockpit
npm run validate:r6-cockpit:master

# Optional 3840×2160 review raster. The 1602×1002 native sources do not become
# native 4K footage when rendered at 2×.
npm run render:r6-cockpit:4k
npm run validate:r6-cockpit:master:4k
```

The manifest `r6-cockpit-props.json` binds the two source hashes, exact MSI and
executable identities, candidate/state receipt hashes, Windows 125% / 120 DPI
source conditions and the disabled evidence claims. This remains a local motion
proof. A publishable launch master still needs continuous native footage with a
visible pointer, hover and click response, the real Review → confirmation → Apply
sequence, native 4K source capture, privacy review and normal-speed human playback.

The retained 1080p proof is
`D:/pytxo-beta-lab/cockpit-beta-candidate-r6-20260921/pytxo-r6-cockpit-proof.mp4`,
3,888,454 bytes, SHA-256
`e47e651c88ac32a68040b124151d3018cb675d3986ba5f21525bf98e4aa87fe8`.
Exact-asset validation, TypeScript, composition discovery, full master validation,
full decode and six-frame contact-sheet inspection pass. Receipt SHA-256 is
`069f978c2593f5c718920c3297b34fec15caa4ae5be920206c4eeeaacb36a292`.
The matching 3840×2160 delivery raster is 9,639,231 bytes, SHA-256
`29cb57c8b47cbb2543c5ed7b4f1f6dab7a692558e0e508138e92ecf857f08f5c`.
Its full decode and original-raster midpoint inspection pass. The vector overlays
are 4K, while the embedded 1602×1002 native frames remain upscaled stills rather
than native 4K capture.

## Prior R3 beta candidate and recording boundary — September 21

The prior native recording target was the unpublished 1.2.2 R3 beta candidate
under `D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/`:

- MSI SHA-256 `8b8e666b0354616c4f2c082917dd184a9aee8846614eb411444538be88f27883`.
- Extracted EXE SHA-256 `ffc6dccd75cacf193b46af4bbae9fed329c4f5ab8d9e4f457f754213a32154bc`.
- The 27 recorded runtime source identities still recompute with zero mismatches.

This exact executable has been launched and inspected at the host's actual 125%
Windows display scale. A genuine Codex mission completed Work → Review → confirmed
Apply in the disposable `real-codex-mission-r3-20260921` fixture; its combined and
post-Apply test runs both passed 7/7. The 24-worker, eight-wave canvas fits at 39%
with the minimap visible, and the redundant bottom commit rail is absent. Exact
native stills are retained beside the candidate:

- running `97ee5b0167060595305f03cec24facb0446918fbde68ed734b632f30493174d7`;
- completed `0201a133d2b2229e52a60ce2af25fafbffaea216846a29d731aa211ab18e2996`;
- Review `dbba16b8b44e0ec42d7c45f4e891ee9e1c09945a2e37d77c9e7ac072ccd44ff0`;
- applied `ca5612548a09b21b6edbb89c906310c69277133c4db0a372dc7aed07a2c88e6b`.

These stills prove those observed surfaces. They are not continuous footage and
carry no pointer, hover, click, transition, or elapsed-time proof. The refreshed
`public/product/*` files are deterministic browser fixtures of the matching UI;
they remain interface references rather than native run evidence. The historical
Remotion compositions below are still bound to their original September 8 inputs.
Do not relabel them as R3 or infer native 4K detail from an upscaled render.

## R3 premium capture plan

Record a short native sample first. It must show one pointer, a real hover response,
a click response, readable text, and uninterrupted motion before recording the
full journey. Keep the final edit near 45–60 seconds, put the app on screen inside
two seconds, and use at least 85% continuous native product footage.

| Edit | Recorded action | Treatment |
| --- | --- | --- |
| 0–3s | Establish exact R3 Work and Chroma Aperture. | Whole window, one restrained product label. |
| 3–10s | Enter one bounded request in New Work and select the ready Codex CLI. | Keep the pointer visible; hold project and agent controls long enough to read. |
| 10–17s | Build and inspect the real plan before starting. | Show recorded paths, checks, worker count, and dependency shape. |
| 17–27s | Observe execution on the canvas; select a worker and open Details or Output. | One gentle canvas move; let state-driven ASCII motion carry the transition. |
| 27–38s | Open Review and inspect prepared bytes plus combined checks. | One useful zoom; preserve package identity and readable diff evidence. |
| 38–49s | Open the confirmation and deliberately Apply. | Keep confirmation → action → receipt continuous at 1×. |
| 49–58s | Show the applied result and History record. | Disclose any restart cut; end on recorded outcome. |

Use only the ASCII states the product actually rendered: ambient Aperture, planning
scan, running worker pulse, candidate convergence, and settled terminal symbols.
Do not add synthetic workers, animated dependency traffic, decorative terminal
streams, or a speed claim. Keep overlays sparse and use Chroma Aperture color only
to explain focus, execution state, and the repository boundary.

## R3 review storyboard

`PytxoR3Storyboard` is a 24-second review composition built from the four exact R3
native stills above. It establishes the premium motion system before continuous
capture exists: a sparse operator briefing, 12 fps ASCII Aperture, full-resolution
cockpit evidence, a four-step Execute → Candidate → Review → Apply rail, and short
12-frame dissolves. The disclosure `R3 storyboard · exact native stills · motion
capture pending` remains visible throughout. The disposable validation path in the
Review and applied stills is covered by two source-coordinate masks that share the
same zoom transform as their source images.

```powershell
cd apps/demo-video
npm run validate:r3-storyboard
npm run render:r3-storyboard
npm run validate:r3-storyboard:master
# Optional 3840×2160 review raster; the still inputs remain 1282×802.
npm run render:r3-storyboard:4k
npm run validate:r3-storyboard:master:4k
```

The retained 1080p review film is
`D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/pytxo-r3-storyboard-review.mp4`,
SHA-256 `d57c73c2ecfaa9e31abd05c10da1bbd058e845b2cd527866a62490211aa9d683`.
Exact asset hashes, mask bounds, full decode, contact-sheet inspection and
transition inspection passed. The exact encoded file also completed a full 1× local
agent playback; normal-speed human playback remains open. The earlier `d8f81a68…`
render was superseded after
review found that its privacy masks did not share the screenshot zoom transform.
This is a motion-design storyboard, not continuous native interaction, pointer
evidence, elapsed-time evidence, native 4K capture, or a publishable launch master.
Replace the still scenes with newly captured R3 clips; do not remove that boundary
merely because the composition renders successfully.

## 4K output and source quality

The host display used for R3 acceptance is 1920×1080. Remotion `--scale=2` produces
a valid 3840×2160 encoded raster and keeps vector overlays crisp, but it cannot
recover native UI detail from 1920×1080 or 1282×802 source media. A true 4K product
master requires a fresh native capture on a 3840×2160 target, ideally with Windows
at 200% scaling so interface text remains readable, followed by privacy and pointer
inspection at the original source resolution.

The checked-in 4K commands intentionally say `historical`: the current composition
still presents the September 8 evidence film. They verify the export path without
claiming a current demo.

```powershell
cd apps/demo-video
npm run render:historical-proof:4k
# After rebinding and reviewing a complete composition:
npm run render:historical-silent:4k
npm run validate:historical-silent:4k
```

The proof command renders frames 0–29 at 3840×2160, H.264 CRF 16, yuv420p,
BT.709, 30 fps, concurrency 1, then performs a complete decode. Use
`node scripts/render-4k-clip.mjs <from> <to> <name.mp4>` for other reviewed ranges;
the range is inclusive and the filename must contain `historical`. For the eventual
R3 edit, preserve the same encoding and
single-worker render settings, bind every native clip hash and cut, inspect the
confirmation boundary frame by frame, and validate the entire master before any
publication review.

The retained one-second proof has SHA-256
`78a81600ae8b6d247d3246f993aa637acb15eb0b84cf435916f1c5b8919d20c7`.
Its machine-readable receipt is
`D:/pytxo-beta-lab/cockpit-beta-candidate-r3-20260921/demo-4k-pipeline.json`.

## Historical capture checkpoints

The September 10 capture target was packaged EXE `e0eeb950…` / MSI `9676d38f…` under
`target/astra-native-finish-20260910/`. All 361 frozen inputs match. Build, packaged
import inspection and affected checks passed; native control is paused after
physical Escape. No launch, mission or recording of this exact package exists.
Use its prepared launcher only after explicit native resumption. The game-like
mockup and Focus/Control design are proposals, not demonstrated app features.

The preceding 6acc native mission completed Cancel, exact Apply and restart with
11 project tests and 26 independent checks. Its raw 1,200-second recording fully
decodes; Review/Apply intervals were inspected, with a confirmed Git startup
interruption retained in the evidence. Raw restart background stays private.
The miniature-frame black-content warning was withdrawn after full-resolution
inspection. The private 90-second proof film is complete at
`target/astra-review-20260910/native-video/out/pytxo-native-review-proof-6acc.mp4`,
SHA256 `92ad0721a4a31797911aa16bb6d138b5ca3ee04f2000e290ed757eeb1208a2a0`.
It has 84 seconds of original-speed footage and a labeled six-second restart still;
Apply is uninterrupted at output 60–84 seconds. Typecheck, render, full decode and
representative frame inspection passed. Normal-speed human playback is unverified.
`tooling/benchmarks/results/astra-native-review-video-2026-09-10.json` binds the edit
and QA to this private output. It cannot certify e0ee or final-build smoothness.
See `tooling/benchmarks/results/astra-native-review-2026-09-10.json` and
`docs/01-projects/astra-native-finish-2026-09-10.md` for exact scope and identities.

September 10 update: the private 94-second f968 native walkthrough is complete:
`target/astra-ux-20260909/native-video-followup/out/pytxo-native-apply-f968.mp4`.
It contains 88 seconds of actual 1× footage and a labeled six-second later restart
still; Apply is uninterrupted. The netsh startup defect is visible and labeled.
Decode/typecheck/frame QA passed; normal-speed human playback is unverified.
Its exact records are `astra-native-menus-2026-09-10.json` and
`astra-native-menus-video-2026-09-10.json` under `tooling/benchmarks/results/`.

The query repair is now packaged as 8910d153…, but physical Escape paused native
control before it launched. Fresh native capture still needs explicit resumption.
The old 073 excerpt and Aperture film below remain historical. See root DEMO and
CHECKPOINT for current identities, evidence limits and the next action.

The **Aperture checkpoint film** is a separate 58-second, 1920×1080, 60 fps
composition. Its optical graphics explain the recorded task graph; its review,
Apply and result scenes use real native WebView MP4 recordings from the local
MSI whose executable SHA-256 starts `c77b8418cf`. The recording excludes OS chrome
and cursor. Sixty frames per second is the encoding cadence, not a measured app
performance claim.

```powershell
npm run typecheck
npm run validate:aperture
npm run test:aperture:privacy
npm run render:aperture
npm run validate:aperture:master
```

The output is `out/pytxo-aperture-native.mp4`. `aperture-props.json` binds the
composition to the exact plan, package, clip hashes, normal-speed intervals and
observed post-state. The asset gate checks those values against
`tooling/benchmarks/results/astra-aperture-native-2026-09-08.json` and the recorded
plan. Review removes 41 seconds at source-clip second 9, labeled on screen; Apply
is one uninterrupted 11-second interval. The separate result clip was captured
after restarting the same executable. The source run passed 11 repository tests
and 26 independent checks after Apply. Host execution, local UI verification and
MSI dependency inspection do not close CI, clean-install or public-download gates.
The proposed Focus/Control customization is absent from this evidence film.

Preserved preparation note for the earlier Desktop MSI (`26ef95bc…`, packaged
executable `073a1a30…`, 359 source inputs): it included the background-readiness
console repair and still needed its own recording at that checkpoint. The
predecessor fcd reached onboarding/Work before a
physical Escape stop; its raw static capture is diagnostic, not a motion demo.
Both the e775/7abe polish
and this film's c77 build are earlier checkpoints. Use the current capture plan
at the top of [DEMO.md](../../DEMO.md): actual native interaction, visible pointer,
hover/click/scroll responses and restrained camera movement. The existing
cursor-free c77 clips cannot supply missing pointer telemetry. Preserve this
composition and its evidence; create a separately bound edit after new capture.

The reviewed redacted master is 7,696,333 bytes, SHA-256
`925ddfcf60d94f83c75d3ec86755cdc765036d3899b08410a5dfce6a734400e2`.
Its [provenance record](../../tooling/benchmarks/results/astra-aperture-demo-2026-09-09.json)
includes all 26 inspected encoded frames, format validation and complete 1×
machine playback with 10 dropped frames out of 3,480 and no media errors.
It remains a local checkpoint film, not a final polished-build launch demo.

`privacyEdits` is an editorial layer separate from the unchanged Bench record.
Each clip's masks are bound to its SHA-256 and use source-frame intervals and
pixel coordinates, so they move with the film's camera. Every mask says “Host
path redacted.” Inspect the masks at UI transitions and confirmation boundaries;
schema checks establish bounds and identity, not complete visual coverage.
The three original MP4s under `public/product/aperture/` are ignored local inputs
and must remain private. Restore the exact hash-matching captures before a local
render; a checkout alone intentionally does not contain those raw host paths.
Only the reviewed redacted master is a potential distribution artifact, subject
to publication approval. Preserve the original master before replacement.
Current coverage is limited to the three zero-trim, 60 fps, 1× sources. Before
introducing fractional trims, align mask timing with the rounded video trim frame;
before any source or camera change, repeat coverage and boundary inspection.

## Earlier still-based film

Exact 52-second, 1920×1080, 30 fps Remotion evidence film of the September 8 real
Codex mission: three scoped tasks, two waves, and at most two concurrent workers.

`mission → combined candidate receipt → explicit confirmation → committed Apply → post-state`

This is an edited presentation of unmodified native element captures, not a live
screen recording or an elapsed-time comparison. The source run, exact reviewed
package and post-Apply checks are recorded in [DEMO.md](../../DEMO.md) and
`tooling/benchmarks/results/astra-final-native-2026-09-08.json`. The film does not
claim an OS-wide sandbox, comparative win, installer proof or Beta availability.
The worker was the real Codex CLI. Test automation operated the native review
and explicit confirmation; this is not an external human-user usability study.
The asset gate compares each image's hash with its original in `docs/_attachments/`.
It also requires a passing independent post-Apply test process. The successful
rehearsal used explicit final-state guidance in the documentation task editor;
the exact override and prior failures are retained in the reproduction fixture
and records. A ready earlier package was withheld after semantic review even
though its tests passed. The product did not detect that prose contradiction.

The master never burns subtitles. Publishing captions and the reviewed transcript live in
`publishing/`. The poster is 1920×1080; native evidence crops retain their original
dimensions and are placed inside the full-HD composition. Generated
video, narration, music, cues, contact sheets, and QA stills are ignored.

## System media tools

Install full builds of `ffmpeg` and `ffprobe` and make both available on `PATH`. Rendering and
release QA use the system binaries for BT.709 finalization and the `loudnorm`, `silencedetect`,
`blackdetect`, and `freezedetect` filters. Confirm both prerequisites before rendering:

```powershell
ffmpeg -version
ffprobe -version
```

## Silent visual master

```powershell
cd apps/demo-video
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
```

The silent master is `out/pytxo-demo-silent.mp4`. It is rendered as H.264 at CRF 18 and
yuv420p, then stream-copied through the finalizer to stamp BT.709 space, transfer, and primaries.
It contains no audio stream.

The silent edit intentionally holds each evidence state for reading. After a
12-frame entrance, scenes hold until seconds 4, 13, 27, 39, 47 and 52; Apply
changes from the captured confirmation to the applied receipt at second 33.
The freeze scan reports these holds for inspection; they are not playback stalls.

## Narrated master

Read `VOICEOVER.md` and `LICENSES.md` first. The narrated pipeline requires one continuous
ElevenLabs narration track and the licensed Modern Chillout bed and certificate.
Regenerate narration from the current `VOICEOVER.md`; older audio does not match
this recut. No simulated click or interface sound is added to the captured states.

```powershell
npm run voiceover
npm run render:narrated
npm run validate:narrated
```

The narrated master uses H.264 CRF 18, yuv420p, fully stamped BT.709 metadata, and AAC at
48 kHz / 256 kbps. Missing credentials or media fail with a complete, actionable asset list; no
provider, voice, music, or cue is substituted.

Narrated validation is a release gate. Integrated loudness must be between -18 and -14 LUFS,
true peak must not exceed -1 dBTP, leading or trailing silence longer than 1 second is rejected,
internal silence longer than 2 seconds is rejected, and full-program silence is always rejected.
The thresholds and parsers have deterministic coverage in `npm run test:audio-qa`.

`publishing/pytxo-demo-en-provisional.srt` is an external, provisional accessibility artifact.
It is never burned into the master. After the approved continuous narration is generated and
reviewed, retime every cue against that exact audio before release; the provisional timings are
not publishable.
