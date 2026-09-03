# Commit-boundary release demo fixture

`prepare.ps1` creates a new committed session under `target/release-demo/` and
proves its dependency-free baseline tests pass. It never resets a previous
session.

The included adapter is deterministic and local. It stands in for a coding
agent so a recording can prove Pytxo's actual isolation, verification, review,
Apply, and evidence behavior without depending on provider latency or model
variance. Do not describe the adapter as an AI-generated result.

The operator runbook and exact commands are in the repository-root `DEMO.md`.
