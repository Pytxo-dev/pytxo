# CHECKPOINT — Phases 60+

**Updated:** 2026-07-07  
**Version:** 0.3.5  
**Goal:** Enterprise GA ops + moat hardening + Deck polish (Phases 60–67)

## Current state

- Phases 60–67 implementation landed in monorepo (code + docs).
- Enterprise GA checklist items addressed in code; production ops (migrations, key rotation) remain operator tasks.
- Competitive benchmarks: run `tooling/benchmarks/overlay-vs-worktree.ps1` on pinned hardware to refresh `docs/06-product/competitive-benchmarks.md`.

## Next operator actions

1. Apply Link migrations through `005_seats.sql` on production Postgres.
2. Set `LINK_REQUIRE_AUTH=1`, rotate `LINK_ADMIN_KEY`, configure `NEXT_PUBLIC_LINK_URL` on Vercel.
3. Run `tooling/scripts/go-live-smoke.ps1` with `LINK_ADMIN_KEY` + `PYTXO_ORG_ID`.

## Key paths

| Phase | Focus | Paths |
|-------|-------|-------|
| 60 | Enterprise GA | `services/pytxo-link/`, `apps/web/`, `tooling/scripts/go-live-smoke.*` |
| 61 | Benchmarks + docs | `docs/06-product/competitive-benchmarks.md` |
| 62 | Blast overlay default | `crates/pytxo-runner/src/blast.rs`, `[blast].prefer_kernel_overlay` |
| 63 | Deck 3D v2 | `apps/desktop/src/components/topology/TopologyScene3D.svelte` |
| 64 | Galaxy HITL | `crates/pytxo-runner/src/hitl_gate.rs` |
| 65 | Windows network | `crates/pytxo-runner/src/network_isolation.rs` |
| 66 | Modular projects v2 | `crates/pytxo-store/src/catalog.rs` |
| 67 | MCP v3 | `crates/pytxo-runner/src/mcp_hub.rs` |
