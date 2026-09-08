# Pytxo product demo

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
