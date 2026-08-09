# Pytxo 52-second demo narration

Direction: calm technical authority; conversational, precise, and understated. Read the
paragraph as one continuous track. Do not split it into scene clips, paraphrase it, or add
spoken calls to action. Pronounce **Pytxo** as “PIT-so”.

<!-- NARRATION_START -->
One mission can involve several coding agents. Pytxo plans who owns each path, maps dependencies, and starts work in isolated workspaces. Before anything reaches your repository, Run Review shows the base revision, execution plan, permission profile, enforcement evidence, and every addition, edit, and deletion prepared for Apply. Reviewed Apply is limited to Orbit and Galaxy, inside one execution domain and one repository root. Approve once, and Pytxo applies that reviewed change set as one crash-recoverable operation. If the checkout moves, Pytxo stops and asks you to review again. Operations shows the decision, ownership, and outcome, not a wall of terminal noise. Keep the agents you already use. Add Pytxo.
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
