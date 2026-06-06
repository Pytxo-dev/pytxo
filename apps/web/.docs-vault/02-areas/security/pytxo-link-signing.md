---
title: Pytxo Link cryptographic verification
slug: pytxo-link-signing
status: active
tags: [security, remote]
audience: [human, agent]
layer: security
created: 2026-06-02
updated: 2026-06-02
related: [[regex-sanitization]], [[hybrid-execution]]
---

# Pytxo Link cryptographic verification

**Pytxo Link** enables P2P remote access (Pro tier). Critical actions—e.g. **Approve & Write** from a phone or secondary device—require **cryptographic verification**.

## Flow

1. Remote device holds a local key pair.
2. Action payload is signed.
3. Desktop host verifies signature against its local trust store.
4. Only then is the `portable-pty` stdin pipe unlocked for the approved command stream.

## Threat model

Prevents malicious or spoofed remote streams from executing shell commands without an explicit, verified approval path.

Part of **Sovereign Shield** with [[regex-sanitization]].
