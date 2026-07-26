---
title: Marketing demo video shot list
slug: demo-video-shot-list
status: active
tags: [guide, marketing, release]
audience: [maintainer]
layer: guide
created: 2026-07-26
updated: 2026-07-26
related: [[release-workflow]]
---

# Marketing demo video shot list

Screen recording for pytxo.com / YouTube. Target length: **90–120 seconds**.

## Prep

1. Install current Desktop + CLI (`pytxo --version`).
2. Settings → Appearance: **Void** skin, **Teal** or **Spectrum** accent (readable on camera).
3. One trusted demo repo with 2–3 agents in `pytxo.toml`.
4. BYOK key set (OpenRouter is enough); Settings → Providers shows Configured.
5. Capture: 1920×1080, 30fps (OBS or Windows Game Bar). Mic optional; VO can be added in edit.

## Shot list

| # | Duration | Scene |
|---|----------|--------|
| 1 | 8s | Open Desktop → Operations (clean idle). Brand visible. |
| 2 | 15s | Dry-run then run 2–3 agents (shell or Desktop Flow). |
| 3 | 20s | Focus / topology: graph + token-savings cue. |
| 4 | 20s | Galaxy approval: open HITL card → approve or deny. |
| 5 | 12s | Settings → Appearance: flip skin/accent (shows polish). |
| 6 | 10s | Settings → Providers: key status (no secrets). |
| 7 | 8s | Close window → tray; reopen from tray. |
| 8 | 8s | End card: `npm i -g pytxo` + pytxo.com/download |

## Edit notes

- Hard cuts; short captions only (product verbs, not slogans).
- No fake multi-terminal wallpaper shots.
- End on download CTA, not a feature dump.

## Host + site

1. Upload to YouTube (public or unlisted) **or** host MP4 (R2 / Cloudflare Stream).
2. Set `NEXT_PUBLIC_DEMO_VIDEO_URL` on the web deploy (YouTube watch/embed URL or direct `.mp4`).
3. Marketing page Demo section reads that env; leave unset to hide the section.

## Do not film

- Real API keys, tokens, or private repo paths.
- Internal Phase/ADR jargon overlays.
