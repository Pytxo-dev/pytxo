# Approval risk demo

A tiny dependency-free project for trying Pytxo with a real, reviewable outcome.

The starter classifies file changes as routine or review-required. Its baseline
tests pass before an agent changes anything.

## Baseline

```powershell
npm test
```

## First mission

Paste this into Pytxo Desktop → Flow:

> Add concise risk summaries for network and destructive command changes in `src/risk-policy.mjs`;
> add regression tests in `test/risk-policy.test.mjs`;
> document two examples in `README.md`. Keep the existing `classifyChange` API.

The mission deliberately spans implementation, tests, and documentation so the
plan can show explicit path ownership, dependencies, isolated execution, and
verification.
