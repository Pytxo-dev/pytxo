# Pytxo v1 demo voiceover

Generate each line as a separate ElevenLabs clip. Keep the filenames below so the Remotion
composition can place every clip on its scene.

Direction: calm technical authority; conversational, precise, and understated. Do not use a
movie-trailer read. Aim for 130–140 words per minute with a short pause after the first sentence
of each clip.

Recommended baseline: `eleven_multilingual_v2`, stability `0.58`, similarity `0.78`, style `0.18`,
speaker boost on. Choose a medium-low voice that sounds like an experienced engineer explaining a
system.

| Time | Filename | Voice line |
|---|---|---|
| 00:00–00:07 | `01-mission.mp3` | One repository. Several coding agents. The hard part is no longer generating code. It is controlling the work. |
| 00:07–00:14 | `02-agent-sessions.mp3` | Pytxo keeps Codex, Claude Code, Cursor, Gemini, and OpenCode where they belong. Each vendor owns its own session. |
| 00:14–00:23 | `03-flow.mp3` | Open your project, or create the guided example with no API key. Describe the outcome, then inspect the plan before anything runs. |
| 00:23–00:32 | `04-race.mp3` | Race Shield maps path claims and dependencies into execution waves, so overlapping edits cannot run in the same stage. |
| 00:32–00:40 | `05-blast.mp3` | Blast Shield gives every agent an isolated workspace. The primary checkout stays unchanged until a human approves the result. |
| 00:40–00:49 | `06-operations.mp3` | Operations compresses terminal noise into state: running work, agents, path locks, approvals, isolation, cost, and the exact run needing attention. |
| 00:49–00:57 | `07-approval.mp3` | High-risk actions arrive with requester, workspace, consequence, and run evidence. Approve and apply, or deny and discard. |
| 00:57–01:06 | `08-benchmarks.mp3` | In this reproducible local harness, Signal reduced structural bytes by eighty-two point eight percent across one hundred eighty-five files, and five of five isolated agents passed. |
| 01:06–01:13 | `09-result.mp3` | Keep your tools. Keep control. Turn one engineering mission into one safe, reviewable result. That is Pytxo. |

Pronunciation:

- **Pytxo**: “PIT-so”
- **Codex**: “CO-decks”
- **OpenCode**: “open code”

Place the clips in `public/voiceover/`, then render:

```powershell
npm run render:voiceover
```

Or generate all nine clips from the canonical JSON:

```powershell
$env:ELEVENLABS_API_KEY = "<rotated-key>"
$env:ELEVENLABS_VOICE_ID = "<voice-id>"
npm run voiceover
```

Never commit generated audio or credentials.
