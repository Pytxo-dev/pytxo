# Pytxo product demo

The edited film below predates the Sep 6 combined-candidate verification changes.
For current Beta proof, use the real one-worker Codex mission, native exact-diff
Review/Apply captures and recorded post-state checks in [DEMO.md](../../DEMO.md).
This older film is not evidence for the v3 candidate receipt. Recut and review it
against the current product before presenting it as the Beta launch master.

Exact 52-second, 1920×1080, 30 fps Remotion demo built from truthful Pytxo Desktop captures:

`mission → Work ownership → reviewed evidence → guarded Apply → verified outcome → Pytxo`

The master never burns subtitles. Publishing captions and the reviewed transcript live in
`publishing/`. The checked-in poster and all three product screenshots are 1920×1080. Generated
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

## Narrated master

Read `VOICEOVER.md` and `LICENSES.md` first. The narrated pipeline requires one continuous
ElevenLabs narration track, the licensed Modern Chillout bed and certificate, and all three
approved interface cues.

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
