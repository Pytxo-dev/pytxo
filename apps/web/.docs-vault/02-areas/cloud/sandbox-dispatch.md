---
title: Cloud sandbox dispatch
slug: sandbox-dispatch
status: active
tags: [cloud, operations]
audience: [human, agent]
layer: cloud
created: 2026-06-02
updated: 2026-06-02
related: [[hybrid-execution]], [[delta-sync]]
---

# Cloud sandbox dispatch

When a swarm task exceeds local capacity or policy, the Rust core **offloads the execution sandbox** to a dedicated **Pytxo Cloud Sandbox**.

## Steps

1. Developer runs Pytxo locally with **BYOK** credentials.
2. User (or policy) triggers offload for compile/test-heavy work.
3. Local core provisions remote sandbox; secrets for LLM calls remain governed by BYOK and [[regex-sanitization]] rules for file payloads.

## Local machine

Stays cool and quiet; telemetry streams to the Reality Deck.

Parent: [[hybrid-execution]].
