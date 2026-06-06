---
title: Contributing
---

# Contributing

Pytxo is open source under the **MIT License**. Contributions welcome via GitHub pull requests.

## Development setup

```bash
git clone https://github.com/Pytxo-dev/pytxo.git
cd pytxo
cargo build --workspace
```

Run smoke checks:

```bash
./tooling/scripts/smoke.sh
# Windows: ./tooling/scripts/smoke.ps1
```

## Guidelines

- Route orchestration changes through the three moats where applicable
- Add tests for scheduling, worktree, and store behavior
- Update `apps/docs/` when user-facing behavior changes (not only the internal vault)
- Follow existing Rust formatting (`cargo fmt`) and CI

## Pull requests

1. Fork and branch from `main`
2. Keep commits focused
3. Ensure `cargo test` and CI pass
4. Link related issues or ADRs in the PR description

Full policy: [CONTRIBUTING.md](https://github.com/Pytxo-dev/pytxo/blob/main/CONTRIBUTING.md) on GitHub.
