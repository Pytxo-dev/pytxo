# Pytxo Desktop film: storyboard (2026-10-17)

Replaces the 1.3.1 CLI land film (`PytxoLand`). Target: about 60 s, 1080p,
no voiceover, cut to the beat of Matt's chosen track, ASCII title system, real Pytxo
Desktop 1.3.2 footage for every product shot.

## Why the CLI film did not convince

- No value in the first 10 s. It opens on "Both branches passed" with no
  context, then spends ~12 s on `npm i -g` and install warnings.
- A wall of 12 px terminal text at 1080p. The decisive lines (BLOCKED,
  `ReferenceError`, LANDED) are the same size and weight as everything else;
  zooms frame whole paragraphs rather than the one line that matters.
- Captions are tiny grey text under the frame and repeat for 10+ s
  ("Fix it on one branch." ×4), so the edit reads as idle.
- Typing and waiting are most of the runtime. It shows a tool working, not
  what the viewer gets.
- No product. It is a terminal, so nothing says "this is a polished product".
- Nothing marks the payoff: LANDED appears like any other line, and the music
  sits under the edit instead of cutting it.

## What premium launch films do (sources below)

- Product in the first 2–3 s; one job; 30–65 s; no narration (Linear for
  Agents 0:33, Cursor 2.0 1:04).
- Masked type reveals, long holds so each idea lands, slow camera drifts over
  dark UI, sharp-in / long-settle easing, no bounce.
- Claim, then proof within seconds; cut any beat that does not pay off.

## Value story

Problem: agents now work in parallel, and their work is checked one branch at
a time. Each branch passes; together they break. (Real: on 2026-10-15 Claude
Code renamed `toVal` in lukeed/mri while Codex added a call to it. Both passed
alone; combined, `ReferenceError: toVal is not defined`.)
Merge tools tell you whether branches *merge*; agent workspaces (Conductor,
Vibe Kanban, Sculptor) isolate and diff. None of them checks the combined
change and then refuses to write anything that moved.

Payoff, all in Desktop: one request → a plan where every file has one owner →
agents from different vendors run in isolated copies → Review checks the **combined**
change → the repo moves after review and Apply **refuses** → refresh → Apply
writes exactly the reviewed files → History keeps the receipt.

## Shot list

| # | Time | Screen / action | Camera | On-screen line |
|---|------|-----------------|--------|----------------|
| 0 | 0:00–0:04 | ASCII aperture resolves from glyph noise | locked | "Your agents ship in parallel." → "Each branch passes." |
| 1 | 0:04–0:09 | ASCII problem card (type only, no fake UI) | glitch burst on the drop | "Together, they break." + `ReferenceError` (source line: lukeed/mri, 2026-10-15) |
| 2 | 0:09–0:16 | Work → New work: paragraph request typed; Split with Claude Code → "proposed 7 tasks, each owning its files" | drift down, zoom 1.6× on the proposal line | "One request." |
| 3 | 0:16–0:22 | Team ticked (OpenCode joins the Claude Code lead) → Build plan → Review plan | drift along step cards | "Every file has one owner." |
| 4 | 0:22–0:32 | Run → Fleet (in-app ASCII radar), time-lapse | zoom on fleet rows turning `passed` | "Isolated copies. Your repo untouched." + "sped up N×" |
| 5 | 0:32–0:42 | Review: summary rail (agents → change → checks → project), scroll one diff | zoom 1.5× on "Checks · all passed" | "Checked as one change." |
| 6 | 0:42–0:50 | A file lands in the repo after review → Apply → refused, "changed since review" | snap zoom on the refusal; music drops out | "Something moved. Nothing written." |
| 7 | 0:50–0:57 | Refresh review → Apply exact package → "Applied successfully" | pull back to full window | "Exactly what you reviewed." |
| 8 | 0:57–1:02 | History → run → receipt (four enforcement surfaces, Apply attempts) | slow drift | "Every Apply leaves a receipt." |
| 9 | 1:02–1:08 | ASCII PYTXO wordmark | locked | "The landing layer for coding agents" · pytxo.com · Desktop 1.3.2 |

Captions sit in the frame margin (the app plays at ~90 % inside an ASCII
field), never over the UI being shown. Every shortened wait carries "sped up
N×". A closing provenance line: real Desktop 1.3.2 run, live agents, waits
shortened and marked.

## Music and beat grid

Track: blksmiith, "sr20det coldrage" (Matt's choice, replacing PLENKA "When
You Find Me"; neither is cleared for commercial use by this repo, so the file
stays out of git in `public/desktop/`). `scripts/beats.mjs` measures 175.25 BPM,
first beat 0.070 s, intro 0–4.86 s, drop at 4.863 s (beat 14), a break
82–137.7 s and closing stabs at 218.5 s and 223.3 s (`desktop-music.json`).

- Intro: "Your agents ship in parallel." on beat 0, "Each branch passes." on
  beat 8.
- Drop (4.863 s): "Together, they break." decodes on the hit with a glitch
  burst; the mri evidence line on beat 18.
- Every shot lasts whole bars (1.369 s) and is fully on screen on a downbeat;
  the ASCII wipe takes the beat before it. Camera moves start on beats.
- Anchors: the split proposal, the end of the run, the refusal and "Applied
  successfully" each land on a bar line of their shot.
- The refusal is the only silence: the track cuts out on it and slams back on
  the Apply downbeat one bar later.
- Outro: the main track stops; the song's own closing stabs carry the ASCII
  wordmark and the tagline.

## ASCII title and transition system

Taken from the site's `ascii-art.tsx` (the shipped style): the aperture ramp
` .:-=+*x#%@`, glitch set `01<>/\|+*x#%@`, Chroma glyph colours
`#7ee1ed #b4b9ff #bd9eef #e9bb91`, the wordmark spectrum
`#f04da3 #ff6a4a #f3c64e #79e07b #45dccb`, IBM Plex Mono and Sora.

- Titles: the line is rasterised to a character grid, decodes glitch → ramp →
  crisp Sora over ~12 frames, holds, then dissolves back into the ramp.
- Transitions: a band of ramp glyphs sweeps across; behind it the next shot
  is drawn cell by cell from its own luminance, then resolves to footage.
- Background: a slow, dim ASCII field (aperture sphere) behind the framed app.
- Camera: drifts and zooms on decisive moments only, easing
  `cubic-bezier(0.16, 1, 0.3, 1)`; cursor redrawn from the take's telemetry.

## Capture

One native take of the released 1.3.2 exe (MSI SHA-256 `f290718d…`, extracted,
not installed) with `tooling/acceptance/run-local.ps1 -Film -Live -Split
-Lead "Claude Code" -Team "OpenCode"`: live Claude Code (lead) and OpenCode on
the taskboard fixture, 60 fps, pointer telemetry, isolated `PYTXO_HOME` and
WebView profile. The take drives the real mouse and keyboard for ~5 min.

Codex (expired sign-in), Cursor Agent (out of usage) and Antigravity (no
headless output) were not usable on the capture day, so the film credits only
the two agents that worked. Re-cut after a new take:

```bash
node scripts/desktop-cut.mjs <evidence>/journey "Claude Code and OpenCode"
npx remotion render PytxoDesktop out/pytxo-desktop-1.3.2.mp4 --concurrency=1 --offthreadvideo-cache-size-in-bytes=150000000 --offthreadvideo-threads=1
```

The track is copied to `public/desktop/music.mp3` by hand; it is not in git.

Sources: Impractical, "Introducing Linear Agent — a motion breakdown"
(impractical.ai/launch-videos/linear-agent); Hound Studio, "22 Best Product
Launch Videos (2026)"; agentsroom.dev "Best tools to run multiple coding
agents in 2026"; Xu et al. 2026, arXiv:2607.04697 (agent PR conflict rates,
semantic conflicts unmeasured).
