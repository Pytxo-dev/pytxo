# Pytxo 52-second demo narration

Direction: calm technical authority; conversational, precise, and understated. Read the
paragraph as one continuous track. Do not split it into scene clips, paraphrase it, or add
spoken calls to action. Pronounce **Pytxo** as “PIT-so”.

<!-- NARRATION_START -->
One harness. Three scoped tasks. This recorded Codex job changed code, tests, and documentation in two waves. Pytxo kept the primary checkout unchanged while workers ran. It reran the checks on the exact combined candidate. The receipt distinguishes enforced boundaries from advisory controls. Every diff was inspected in the native app, and Apply was explicitly confirmed. The journal recorded a committed attempt. All three applied file hashes matched the reviewed package, and the repository tests passed. These are edited captures from the packaged Windows app. The Bench record also retains an earlier refused run and a direct worktree comparison. No speed win or OS-wide sandbox is claimed. Keep your agent. Add Pytxo.
<!-- NARRATION_END -->

The generation script reads the exact text between the markers and writes one ignored file:
`public/audio/narration/pytxo-demo-narration.mp3`.

```powershell
$env:ELEVENLABS_API_KEY = "<rotated-key>"
$env:ELEVENLABS_VOICE_ID = "<approved-voice-id>"
npm run voiceover
```

`ELEVENLABS_API_KEY` and `ELEVENLABS_VOICE_ID` are mandatory. The script does not select a
fallback voice or provider. Generated narration must be reviewed for wording, pronunciation,
pace, and scene alignment before `npm run render:narrated`.

`publishing/pytxo-demo-en-provisional.srt` preserves this paragraph verbatim but its cue timings
are provisional. After the continuous narration is generated and approved, retime every cue
against that exact audio before release. Do not publish the provisional timing file.
