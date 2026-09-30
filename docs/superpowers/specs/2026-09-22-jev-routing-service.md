---
title: Sponsored routing service admission and accounting
slug: 2026-09-22-jev-routing-service
status: draft
tags: [routing, service, accounting, privacy]
audience: [human, agent]
layer: cloud
created: 2026-09-22
updated: 2026-09-29
related: ["[[2026-09-22-jev-routing-design]]", "[[2026-09-22-jev-routing-contracts]]", "[[jev-typesafe-research-2026-09-22]]"]
---

# Routing included: a bounded application feature

Current implementation note (2026-09-29): Desktop can review the hosted packet, record local consent, and reconcile an account-bound Link workspace grant and revocation. A staged Desktop HTTP client now issues a workspace/revision-bound Link token and sends the exact bounded request to Proxy in loopback tests; it checks the bound receipt and uses no redirects, ambient proxy or retries. Its constructor rechecks the verified account and reviewed local/remote grant, but no normal Desktop dispatch calls it. Hosted Flow remains `review_only`. A separately compiled test-faults entrypoint can claim an exact reviewed one-task hosted Shadow Flow only with an explicit environment opt-in and injected fake client. Stop and claim are ordered by an exact Catalog compare-and-swap, and dead-owner recovery settles partial qualification and advisor-send journal states without retrying. The proxy's paid-send startup gate remains closed. The dated milestones below describe what existed when each slice was written and are not a current inventory.

This is a proposed service contract. The current managed-inference proxy is not sufficient authentication or accounting for sponsored routing. Keep its existing paid behavior separate.

The current experimental Link slice accepts a signed Clerk session with
configured issuer, audience, user and session IDs; it stores one hashed token
per account in migration 009, caps expiry at five minutes or the source
session's expiry, throttles rotation and supports revocation. Migrations
010–018 add isolated funding, account, operation, output-usage, pinned-rate,
encrypted-recovery, workspace-grant and Desktop-bridge ledgers. Link now
reserves, uniquely claims, settles and conservatively closes
stale operations using one database lock order. The dedicated proxy endpoint
validates only the reviewed coarse packet and can call that protocol in an
injected test. Real paid-send startup is hard-blocked even if its experiment
flag is set. No production flag or funded ceiling was enabled by this work. Current Desktop
consent remains bound to the no-network fixture, so the hosted recipient has
**no valid workspace grant** and must not be activated yet. Completed replies
now have a 24-hour encrypted recovery receipt in Link; a same-packet retry can
recover the answer without another upstream send. The proxy continues
post-claim settlement after a client deadline or disconnection. The local Shadow path does not use this
hosted endpoint, and this is not a hosted routing beta or Live advice.

Store migration v15 now has a separate, recipient-scoped hosted consent row and
nullable recipient/scope columns on the send journal. The reviewed policy may
carry a recipient; omission preserves legacy mission bytes and only denotes
the no-network fixture. A hosted journal request requires a current UUIDv7 ID
and Shadow mode, but these local APIs do not derive packet bytes or grant a
network send. Desktop still has no hosted review/consent action or guarded
client. No hosted grant, token or paid provider call is created by v15.

Current local Shadow reviews name and bind `pytxo-local-advisor-fixture/no-network/v1` as the only permitted recipient. The HTTP Jev transport cannot match that identity and is not wired into production dispatch. A browser-to-Desktop account bridge now exchanges a one-use PKCE code for a 24-hour routing-only credential and checks it against Link. That credential is not a hosted workspace grant. A hosted recipient needs its own versioned destination identity, a new reviewed policy/preview, workspace consent, Link admission and metering, and a fresh security review before any send. Changing the recipient must invalidate older saved Shadow reviews; a request-body digest alone does not authorize a destination.

Catalog v7 now has an unused local identity seam for that future grant: one random 16-byte opaque hosted workspace ID per registered domain. Issuance requires the current Catalog Store path and the immutable pinned local consent Store path/file identity to match caller-supplied physical file evidence in one transaction. A separate read-only lookup retains the old ID for eventual revocation after domain removal; it is not grant authority. The future caller must obtain the file identity from a physically guarded opened Store and recheck it at the grant/send boundary. No Desktop flow calls this seam, creates a hosted grant, or sends a packet yet.

Desktop now has a read-only **proposed hosted packet** disclosure for a current persisted one-task Orbit Shadow review. It reuses the reviewed coarse packet projection and shows the prospective hosted recipient, scope digest, proxy packet schema and template, and packet JSON. It explicitly distinguishes that packet from the future client request envelope (which would add a request ID) and the server-generated Jev body. The saved review and its consent setter remain bound to the no-network fixture; viewing the hosted proposal does not create a hosted review or grant, mint an opaque workspace ID, contact Link, issue a token, or send data. The proposal does not promise exact future network bytes. A shared test fixture now links a real staged Flow proposal to the proxy's actual wire validator without a network send; this is a compatibility check, not hosted admission or privacy proof.

## Request path

1. Desktop obtains a short-lived opaque routing token from a new Link endpoint using a validated real account session. Link binds its hashed token to account, expiry and routing-only scope. Reject anonymous identities, the shared api-key sentinel, client-supplied account headers and development auth bypasses. Reuse verified identity machinery, while explicitly checking its configured issuer/audience and required claims.
2. Desktop sends POST /v1/routing/evaluations to a dedicated proxy handler. That handler requires the routing token regardless of the proxy's general auth settings. Unknown routing subpaths/methods never fall through to provider forwarding.
3. Link atomically authenticates the token and reserves account/global allowance. Proxy-to-Link admission uses service authentication; account identity comes from Link's token lookup, never an arbitrary forwarded header.
4. One proxy owner claims the reserved operation, sends one fixed upstream evaluation and persists its receipt through Link. Desktop receives advice or a typed fallback reason. Local Core separately decides whether a returned result is still relevant.

Do not reuse the proxy's current authorized() function: it checks paid tiers when Link is configured and otherwise can accept any nonempty bearer. Its token buckets are process-local and keyed by bearer, so token rotation/restarts/replicas can bypass an account allowance. These are source findings, not claims about a deployed configuration.

No new standalone service/queue is required initially: dedicated proxy module plus Link-owned transactional tables suffices. Account/global counters and idempotency live in one database transaction. No provider keys reach Desktop; worker keys or vendor sessions never enter this service.

## Versioned wire contract

Request: schema_version, client request_id, decision_kind=initial_demand, question_set_version, and packet. The model and questions are server-owned. The current local packet contains a conservative one-line projection from the reviewed goal, coarse task/check categories and opaque capability-role descriptions. The projection falls back to coarse facts for obvious sensitive or code-like text, but heuristic redaction is not a privacy guarantee. These fields can reveal task type or user context, so a hosted send requires a workspace opt-in and exact-byte preview. It excludes local run IDs, paths, remotes, source blobs, account bindings, executable argv, credentials and skill text. Any broader description field needs a separately reviewed disclosure design and packet-version change.

Response: evaluation_id, request_id, question_set/model versions, typed distribution or unavailable reason, server usage receipt ID, input-token count if known, and usage state. It has no commands, task mutations, monetary authorization or Apply fields. Core binds it to its locally stored packet digest and task/auth/consent revision.

Limit dynamic packet JSON to 16 KiB and the server template to a fixed tested size. V1a uses one question; reject arbitrary additional fields and questions. The client and proxy independently validate schema. No general Jev pass-through endpoint.

Enforce the byte limit while reading, not from Content-Length alone; disable compressed requests initially. Bound upstream response size and reject duplicate JSON keys, malformed numeric fields and unexpected response structure before allocation-heavy processing. Authentication/rate rejection must not log the rejected body.

Proposed limits: 10 new requests/minute/account, 2 in flight/account, 3 million input tokens/account/UTC month, 10 evaluations/mission locally, and a separately funded global ceiling. Invited pilot: at most 100 accounts and $50/month upstream allocation, pending authorization. IP throttling is only a secondary abuse control. Free-account creation does not bypass the global ceiling. No indefinite queue: capacity unavailable returns a reason immediately.

## Reservation protocol and uncertainty

Use integer token counts and integer nano-USD. At the reviewed $0.042/M input rate, one token costs 42 nano-USD. Pin a rate-card revision and retain it in each reservation. [TypeSafe model pricing](https://docs.typesafe.ai/models)

The first draft treated the 64k context limit as a proven billing ceiling. That is too strong. The public docs give context limits and returned usage, but do not explicitly establish a complete billing bound for hidden framing, failed/ambiguous requests or future rate changes. Treat 65,536 input tokens ($0.002752512 at that rate) as a **conservative planning estimate**, not a verified financial guarantee. [Models](https://docs.typesafe.ai/models), [API](https://docs.typesafe.ai/api)

Before sponsored activation, establish a verified maximum billable amount per admitted request under the pinned route, or a provider-supported hard prepaid/spending limit. A tokenizer alone does not prove provider-side framing/billing. If neither exists, only an explicitly authorized risk-bounded pilot is supportable; do not market the local counter as a hard external bill cap.

September 28 provider check: [TypeSafe's current model page](https://docs.typesafe.ai/models) still lists `jev-1.13.0` at $0.042 per million input tokens, with output tokens free, and describes the 64k context limit. Neither that page nor the [API reference](https://docs.typesafe.ai/api) specifies a maximum *billable* token count per admitted request. The [current customer agreement](https://typesafe.ai/legal/mca) says that, without automatic purchased-credit refills, TypeSafe **may** decline a request after credits are depleted; this is not a guaranteed hard spending stop. Keep the paid-send gate closed until the account's actual credit/refill settings and a provider-enforced bound or an explicitly authorized, risk-bounded pilot are verified. The local Link ceiling only limits Pytxo admission.

Proposed request states:

| State | Accounting and retry behavior |
|---|---|
| reserved | Atomically reserve the verified upper bound against both account tokens and global spend; consume rate/concurrency capacity. |
| sending | Unique service-owner CAS marks that an upstream send may have happened. No other replica may resend this operation. |
| completed | Persist response/usage and settle actual tokens/cost atomically; release unused reservation and in-flight slot. |
| rejected_before_send | Positive evidence no upstream send occurred permits releasing the reservation. |
| uncertain | Timeout, lost response or owner crash after sending: keep the upper bound consumed/reserved; no upstream retry. |
| settled_at_ceiling | Close an unreconcilable operation conservatively at the reserved bound, flagged as estimated; never relabel it actual usage. |

If returned usage exceeds the reserve, record full actual usage, debit the ledger and disable new sponsored calls pending investigation. A local reservation cannot retroactively stop provider billing.

V1 uses **zero automatic upstream retries**, including explicit 429/529; use rules fallback and a short circuit. This intentionally simplifies the first draft's layered retry policy. Disable SDK retries. Client delivery retries reuse the same request ID; they may retrieve the same receipt but never start another upstream call.

The client waits at most two seconds. Proxy upstream work has a bounded five-second request deadline; a late result may settle usage, but cannot revive a local task decision. Three transient failures open a 60-second local advisor circuit. Auth/schema faults remain disabled until relevant configuration/version changes. No periodic model polling.

Account and global admission checks include outstanding/uncertain reservations, so “3 million tokens” can be temporarily reduced by conservatively held usage. The month is assigned at admission; settlement stays in that bucket after midnight. A month rollover does not forgive unknown global liabilities.

An aggregate invoice can reconcile totals, not prove which lost request was unbilled. Release uncertain usage only with attributable evidence; otherwise retain the ceiling estimate. Counter/database failure disables hosted admission while local rules continue.

## Idempotency and privacy

Use account + request ID as the unique key and an account-scoped keyed hash of the canonical payload. Different payload under the same ID returns conflict; duplicates return in-flight/completed/expired status. To bound replay storage, use time-bearing UUIDv7 IDs, admit new IDs only within 24 hours of their embedded creation time (five-minute future skew), and reject expired IDs after tombstone deletion. A changed ID is a new billable operation subject to all limits.

Proposed Pytxo retention: no persisted raw packets; encrypted minimal answer recovery for 24 hours; metering receipts and ID tombstones for 90 days. Disable request/response payloads in application, HTTP/APM and upstream error logs. Provider retention and account-deletion exceptions need explicit disclosure. Do not advertise zero retention from these local settings.

Because raw packets are not persisted, a crashed reserved job cannot silently reconstruct and resend them; a matching client retry may supply the packet only if the job is provably reserved and unsent. After sending, uncertainty rules apply. No hidden durable prompt queue.

The user-facing workspace opt-in must name the actual hosted recipient, packet schema and template version. The local Store now records a scope digest with each enabled consent revision and compares it with the reviewed mission before preparing, sending or using advice. Migration v14 disables older unscoped grants. The current scope identity covers the fixed **no-network fixture** recipient, wire template and complete coarse projection source bytes; even a projection-only source edit invalidates an older grant. This conservative binding also changes on comments or formatting in that small file. It is not an opt-in to a future hosted service. A hosted recipient needs its own reviewed scope identity and an actual user-facing consent action. A recipient or packet/template change requires a fresh opt-in. Revocation must fence new local requests, prevent later Link claims, and make pending replies unusable locally even if the local clock moved backward. It cannot retract bytes already delivered upstream. Link currently checks grant and token in the claim transaction, but a claim that commits *before* revocation can still be followed by the proxy's upstream send. Treat that operation as in flight and disclose that it may finish after the revoke action; do not promise zero post-click disclosure. The local `jev:<digest>` journal request ID also does not satisfy Link's UUIDv7 admission rule. Store v15 provides the local UUIDv7 identity and recipient-scoped grant, but a trusted Flow client must still derive the actual reviewed packet and scope, reconcile local and Link revoke/status with race and crash tests, and enforce the final send fence before any grant, token or send. Outcome sharing is a separate opt-in. The subsidy check requires account/usage facts, not repository identities or worker results.

Activation checks include wrong-audience/expired/shared credentials, rotated tokens, duplicate requests across replicas, oversize packets, concurrent month/global exhaustion, crash at every send/settle boundary, clock skew, late responses and payload-log inspection. These are planned checks, not evidence already obtained.
