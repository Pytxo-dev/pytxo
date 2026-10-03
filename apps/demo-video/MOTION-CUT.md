# Pytxo Motion Cut

50 seconds / 1920 x 1080 / 60 fps / stereo music and editorial sound effects.
Composition: `PytxoMotionFilm`. Output: `out/pytxo-motion.mp4`.
The existing `PytxoBetaFilm` and silent master are preserved.

## Direction

A precise, connected sequence: split the work, converge the files, stop at
the boundary, then carry the reviewed package through. The design reference
is Notion's editorial clarity and object continuity, not its illustrations,
branding, characters, music, or copied advertising.

Actual product imagery appears immediately. White space, black rules,
large type, restrained green and one refusal accent carry the film. Native
status crops are evidence inserts, never simulated live actions.

| Time | Sequence | Motion |
| --- | --- | --- |
| 0-4 | Pytxo / actual Work view | Product-first reveal and deliberate reframing |
| 4-19 | One request, six tasks | Split, align four parallel lanes, then two ordered handoffs |
| 19-26 | One review package | Seven documents converge into the native review view |
| 26-34 | The stale boundary | The package stops; the music dips; native refusal |
| 34-46 | Recheck, review, Apply | Boundary opens, recorded receipt, actual task-board result |
| 46-50 | Keep the final say | Short typographic close |

## Evidence Contract

Source: the run and immutable capture hashes in `fleet-props.json`.
Six tasks, five vendors, waves 4 / 1 / 1; seven files prepared by four workers.
OpenCode and Antigravity prepared no changes. Six recorded check commands
passed, which is not a claim that every requested feature was implemented.
The result retains the recorded implementation's limitations.

The film is an edited October 2 native-still sequence with explanatory
graphics, not continuous footage or proof of the final candidate's runtime.
No fabricated pointer, task output, elapsed time, or Apply interaction.
Editorial effects are not application notification recordings.

## Audio and Cloud Verification

See `public/audio/cc0/README.md` for primary license sources and
`motion-audio.json` for hashes, gain values and timed cues. The source is
real recorded live-bass music, not generated stock-style music.

The `Demo motion film (private artifact)` GitHub Actions workflow performs
type checking, source-hash/fact validation, audio mixing, Remotion rendering,
and final encoded-master QA. It exports twenty full-size review frames,
the stereo mix, MP4 and machine-readable measurements as private artifacts.
It has read-only repository permissions and no deployment, release or secret
access. No new acceptance testing or rendering runs on Matt's PC.

Run in a disposable/cloud environment with Node 22 and FFmpeg:

```sh
npm ci
npm run typecheck
npm run render:motion
npm run validate:motion:master
```

Automated format, loudness, black-frame and hold scans do not establish
subjective musical quality or replace visual inspection. The final review
must inspect encoded frames and pacing, not only source code or build success.
Windows install, upgrade, native updater and true display-DPI acceptance remain
separate release gates. Linux media rendering cannot close them.
