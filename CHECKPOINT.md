# CHECKPOINT — Pytxo v1.0.0

**Updated:** 2026-07-31
**Version:** 1.0.0
**Goal:** Release the dependable local mission loop across CLI, Desktop, docs,
website, demo, and reproducible evidence.

## Release state

- Flow plans natural-language missions into inspectable task paths and waves.
- Blast-isolated downstream tasks receive successful dependency output before
  starting; the v1 guided mission completed three agents in two waves with all
  regression tests passing.
- Integrations detects vendor-owned Codex, Claude Code, Cursor Agent, OpenCode,
  Gemini CLI, and Aider sessions without exposing account or token data.
- Trust and live-process registries use locked, recoverable writes.
- Signal and control-plane benchmark scripts publish bounded raw JSON with
  explicit non-claims.
- Pytxo Desktop, the public docs/site, the Remotion demo, and the ElevenLabs /
  Screen Studio production scripts are part of the release source.

## Distribution constraint

GitHub-hosted runners are currently account/billing-blocked before job startup.
The v1.0.0 release therefore uses the documented Windows-first fallback: a
fresh Windows CLI and Desktop package, with previous-matrix macOS/Linux CLI
assets mirrored only for install compatibility and labeled as such.

## Key release paths

| Surface | Path |
|---------|------|
| Release notes | `distribution/release-notes/v1.0.0.md` |
| Desktop | `apps/desktop/` |
| Public site/docs | `apps/web/` |
| Demo source | `apps/demo-video/` |
| Guided example | `examples/pytxo-first-mission/` |
| Benchmarks | `tooling/benchmarks/` |
