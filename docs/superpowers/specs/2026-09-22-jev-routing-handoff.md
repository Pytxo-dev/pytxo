---
title: Portable routed attempt handoff manifest
slug: 2026-09-22-jev-routing-handoff
status: draft
tags: [routing, handoff, artifacts, verification]
audience: [human, agent]
layer: execution
created: 2026-09-22
updated: 2026-09-29
related: ["[[2026-09-22-jev-routing-contracts]]", "[[2026-09-22-jev-routing-stress-review]]"]
---

# Portable handoff v1

## Implementation status (2026-09-29)

The experimental local fixture retains a canonical manifest for its dependent child under the exact attempt and capacity owner. Store checks the manifest bytes, reviewed task identity, base and current verified dependency winner at admission and again before launch. The child materializes the predecessor's retained sealed output, not a mutable worker directory. A feature-gated Windows test now runs the producer under a pinned `cmd.exe` adapter and the dependent child under a distinct pinned PowerShell adapter. After the producer passes and releases ownership, the test mutates its old worktree; the child still reads the exact retained seed bytes, passes its frozen check, and reaches Review → Apply. This is local cross-adapter dependency evidence, not qualification of two production harnesses.

The active v1 manifest receiver rejects `repair_input`, `changes`, `evidence` and `notes` until their referenced bytes can be retained and checked under the same owner. Core's repair contract requires `failed_attempt_id` to name a **different, preceding** attempt; a future receiver must also bind that ID to the ledger's exact failed predecessor. The strong ordinal-2 Claude route starts clean from the reviewed base, records its predecessor and failure in the attempt ledger, and receives only the first owned check ID, exit code and exact receipt digest in its private prompt. The receipt is rechecked before admission and launch, but this supplement is outside the portable manifest. It does not forward checker output, import a repair diff, or prove failed-attempt transfer to a different production harness. The broader format and real-harness qualification probe below remain proposed.

“Portable” means a different qualified harness can start a fresh attempt from a Pytxo task contract and frozen bytes. It does not mean resuming a vendor session or transferring arbitrary OS/process state. V1 is local, within one repository/domain and the original reviewed task. No network upload is implied.

## Manifest

Use canonical JSON plus a local content-addressed blob directory, not a vendor transcript or an imported archive. All blob references are digest and byte length; the consumer resolves them through Core's artifact store, never through a path supplied in the packet.

| Field | Meaning |
|---|---|
| schema_version, manifest_digest | Version and identity of this immutable bundle. |
| origin | Domain/run/task/attempt IDs; task, plan and authorization revisions. IDs are local provenance, not authority grants. |
| task_contract_ref | Original goal, constraints, claims, approved dependency/check identities. No newly generated commands are accepted. |
| base | Repository identity, Git revision plus exact starting snapshot digest. A commit alone is insufficient if approved input includes other bytes. |
| dependencies[] | Task ID, winning attempt ID, frozen output digest and verification receipt. Ordered composition follows the approved graph. |
| repair_input | Optional failed attempt ID and scoped change manifest; explicitly untrusted and usable only for this same task. |
| changes[] | Normalized relative path, add/modify/delete, preimage digest, result digest/length, executable mode where supported. Represent rename as delete + add in v1. |
| evidence[] | Core-generated process outcome, quiescence evidence, check IDs/results and sanitized diagnostic blob references; classify each as observed or claimed. |
| requirements | Required receiving capabilities and approved skill/tool bundle references; no permission or billing escalation. |
| notes[] | Optional bounded worker notes/remaining questions, always claimed. Omit hidden reasoning, full chat history and authentication state. |

Ownership stays with the local ledger. A bundle copied elsewhere has content provenance, not permission to run. Retain manifests/blobs while referenced by a live run, task winner or review package; garbage collection requires reference checks. Deleting history cannot silently delete inputs of an active dependent.

## Export/import protocol

1. Stop the prior attempt and confirm owned worker/verifier descendants are quiescent. A returned shell exit or empty process-registry entry is insufficient when descendants may still write. If the adapter cannot establish this, automatic handoff is ineligible.
2. Freeze scoped output after the last writer stops. Reuse existing inventory/path/content safety helpers where their semantics fit; do not reuse an Apply receipt as a handoff approval. Record provenance and content hashes before releasing relevant ownership.
3. Validate path normalization on the receiving OS: reject traversal, absolute/drive/UNC paths, alternate data streams, reserved device names, case collisions, symlinks/junctions/reparse traversal, unsupported modes and out-of-claim changes. Include hidden-file cases. Unsupported submodules/LFS pointer materialization or file kinds reject portability rather than silently changing semantics.
4. Materialize a fresh sandbox from the frozen base and successful dependency manifests. Verify every preimage, blob hash/length and resulting tree identity; never reread a mutable producer workspace as the source of truth.
5. Optionally overlay the validated failed-attempt repair diff. If absent/invalid, explicitly choose clean restart from the last admissible input; record that decision and still consume the second attempt. Do not silently drop failed bytes and call it a successful handoff.
6. Revalidate receiving recipe/binding/skills/tools, effective permissions, budget, remaining attempt count and task revision. Resolve credentials locally. Start a new session with the same task/check contract and a deterministic evidence summary.
7. Verify the new output independently; do not inherit the prior attempt's success label, cost estimate or Apply approval.

Never transfer .git credentials/hooks, vendor session directories, .pytxo control records, secret files or raw environment/command credentials. Baseline project instructions/configuration remain reviewed inputs, but worker-modified AGENTS/skill/MCP/executable-hook configuration cannot become active receiving instructions automatically. Quarantine those modifications as candidate data pending separate review. A legitimate task that needs such configuration changes is outside automatic repair until its instructions are explicitly authorized.

Checks are IDs plus trusted recipes resolved from TaskContract. A worker's suggested command is a claim. Tests changed by the worker may be candidate changes, but they cannot replace the frozen acceptance oracle. Verification occurs against exact proposed bytes in a fresh controlled view.

## Example and compatibility probe

Attempt A changes a settings parser but fails a frozen regression. Core seals its allowed diff and observed failure. Attempt B starts from the original approved base plus successful prerequisites, overlays A's repair diff, and receives the unchanged task and failure evidence. B's successful output becomes the sole task winner; A remains a failed paid attempt in history. The final combined candidate is checked again before Review.

Qualify A-to-B with a real small edit/check task, cancellation, a restart between attempts, missing/corrupt blobs, rejected paths, changed receiving config, unsupported capabilities, a background writer and stale base. Demonstrate both repair-input and clean-restart modes. Same-harness retry qualification does not establish cross-harness portability.

No generative summary, transcript converter, task-DAG rewrite or general package-transfer framework is needed. Start with Core-rendered task/evidence text and local blobs; let worker notes supplement it without authority.
