# Pytxo 52-second demo narration

Direction: calm technical authority; conversational, precise, and understated. Read the
paragraph as one continuous track. Do not split it into scene clips, paraphrase it, or add
spoken calls to action. Pronounce **Pytxo** as “PIT-so”.

<!-- NARRATION_START -->
One mission. One Codex worker. In this recorded run, Pytxo kept three changed files away from the primary checkout until Apply. It checked the combined candidate before Review. The receipt shows one passing test command, and both enforced and advisory boundaries. The exact changes were reviewed in the native app, and Apply was explicitly confirmed. Pytxo recorded a committed attempt. The applied file hashes matched the reviewed package, and four tests passed in the repository. If included inputs change, the candidate needs fresh checks. This is one observed run, not a speed comparison or a sandbox guarantee. Keep your agent. Add Pytxo.
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
