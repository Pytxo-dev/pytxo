# Routing comparison analysis (offline)

`analyze-trial.mjs` is a local analysis tool for the **unexecuted** R0-versus-RJ
protocol in `docs/superpowers/specs/2026-09-22-jev-routing-benchmark.md`. It does
not run workers, contact Jev, export the private routing Store, authorize spend,
or enable routing. `freeze-confirmation.mjs` prepares and checks an offline
held-out assignment schedule. `record-trial.mjs` freezes task/run identity and
assembles local evidence. Neither makes worker or provider calls. The tests use
synthetic data only. No corpus, funded screen,
held-out trial, or Jev benefit is established by this directory.

For confirmation, prepare a private input with `protocol` (`phase: "confirmation"`,
`schema_version: 1`, opaque campaign ID, nonzero uint32 analysis seed and 10,000
resamples), a nonzero uint32 `randomization_seed`, SHA-256 `pins` for the profile
pair, adapter, R0/RJ policies and oracle manifest, and exactly 400 `cases`.
Each case needs opaque case/repository/billing-cohort IDs, SHA-256 snapshot,
reviewed redacted-packet, task-contract and oracle digests, one of the six families, `split: "holdout"`,
`billing_mode: "api"`, and a boolean `planned_advice_opportunity`. Mark it true only when the
reviewed initial task and qualified pair are expected to meet **all** Core Live gates:
an authorized evaluated policy, first-attempt strong default, complete diagnosed
single-component work, both profiles eligible, and the relevant approval and
consent state. Independently check that claim against the
frozen task/qualification evidence before outcomes. It is a pretrial stratum,
not a runtime eligibility receipt. Cases must represent 200 repositories with two tasks
each and one comparable API billing cohort. This prepares a schedule only; it
does **not** authorize API expenditure or a billing-mode change.

```powershell
node tooling/benchmarks/routing/freeze-confirmation.mjs INPUT.json FROZEN.json
node tooling/benchmarks/routing/freeze-confirmation.mjs --verify FROZEN.json
```

The output is a deterministic frozen-schema-v2, 1,600-assignment R0/RJ crossover schedule with
pinned digests. It excludes goals, paths, oracle bytes and worker content, but
**keep it private**: the raw digest of a small, enumerable packet vocabulary can
reveal coarse task facts by dictionary lookup. The study and analysis bundle
also contain those raw packet digests and need the same private handling. Freeze
writes once and will not overwrite an existing artifact. Its
verifier checks structure, pairing, family balance, at least two distinct packet
digests **among planned advice-opportunity cases**, assignment IDs, order and digests.
The report shows the planned opportunity case, repository and packet counts.
The case-manifest digest cannot be reconstructed from the frozen
output; archive and independently register the private input and frozen digest
**before** outcomes are exposed. This is a schedule freeze, not a complete trial
registration: power, permit/budget, exclusions, stopping rules, packet/template
provenance, analysis version, and source-receipt checks remain outstanding.
Digest diversity is necessary, not proof that packets carry useful demand signal
or that the opportunity cohort is large enough for a useful trial. Pre-register
the cohort's expected size, repository spread and power from screening data;
if the fixed trial lacks 80% power for the declared useful effect, report it as
limited-power or register a larger funded trial before seeing outcomes.
A reviewer must verify each opportunity label and packet digest against the exact
local reviewed task projection before the freeze. The analyzer still reports
conditional numbers only; this additional gate cannot authorize live routing.

`PytxoStore::routing_benchmark_trace` (trace schema v3) supplies a scoped Store measurement
projection for one task. A local recorder must pass a private, random 32-byte
key for each frozen assignment; the trace aliases IDs and receipt digests with
that key, verifies Store event/result and admission links, and omits prompts,
credentials, raw advice, and artifact bytes. It also includes every local
advisor may-send journal row for the task, including uncertain or late-result
rows, and rejects a Jev shadow observation without its completed matching send.
Private journal timestamps establish order, but are omitted from the export
because exact wall times could link assignments; a completed local row is not
a provider charge receipt. `decision_links_complete` covers only
Store decision/admission links. V3 additionally aliases five Store mission pins:
task contract, snapshot, qualified profile pair, arm policy, and adapter. It is
not an analyzer observation.

### Assignment-bound recorder

Prepare a private `REGISTRATION.json` **before reading any outcomes**. For a
fixture study it contains the analyzer's `protocol` and `assignments`, explicit
`pins` (`profile_pair_digest`, `adapter_digest`, `r0_policy_digest`,
`rj_policy_digest`), plus one `bindings` row per assignment. For a confirmation
study it instead contains the
complete `frozen_confirmation`, its `registered_freeze_digest`, the original
private `private_cases` used by `freeze-confirmation.mjs`, and the bindings.
The recorder reconstructs the confirmation case manifest and rejects
task-contract, packet, schedule, or policy drift. Each private binding has:

```json
{
  "assignment_id": "assignment_R0",
  "task_contract_digest": "<sha256 of frozen task contract>",
  "scope": { "domain_id": "private_domain", "run_id": "preassigned_run" },
  "task_id": "preassigned_task",
  "alias_key_hex": "<64 lowercase hex characters from a fresh random 32-byte key>"
}
```

Generate each key with a cryptographic random source and retain this private
registration for the matching Store export. Use a distinct run scope for every
assignment so repetitions cannot reuse one execution. The key, raw run/task
IDs, and private cases do not enter `STUDY.json`; it contains their assignment-keyed
aliases and frozen task-contract digests. Archive and externally register the
study digest and, for confirmation, the earlier confirmation freeze digest
before opening outcomes. A local JSON file or file timestamp cannot prove
that chronological step.

Each frozen binding contains `packet_digest`, the assignment-keyed alias of the
case's raw reviewed packet digest. Store exports the same alias for each advisor
journal row; the recorder compares aliases, never an alias to a raw digest.
It also contains `run_pins`: five assignment-keyed aliases of
the raw 64-hex mission digests. The task-contract pin comes from the private
binding (and must match the frozen case manifest in confirmation); the snapshot
pin comes from its assignment; profile-pair and adapter pins come from the
frozen profile/adapter identities; and the policy pin is R0 or RJ according to
the assigned arm. Pytxo Core computes the profile-pair raw digest as canonical
SHA-256 of `[everyday_profile_digest, everyday_binding_digest,
strong_profile_digest, strong_binding_digest]`. Both reviewed profiles must
have the same adapter digest for this comparison. These pinned digests do not
by themselves prove which physical executable or runtime settings launched;
that qualification and its controller receipts remain a separate trial gate.

```powershell
node tooling/benchmarks/routing/record-trial.mjs --freeze REGISTRATION.json STUDY.json
```

For a run already present in that repository's Store, the experimental CLI
target exports one assignment's sanitized v3 trace. The target is excluded
from normal builds. Pass the **private registration**, not `STUDY.json`: only
the registration contains the fresh assignment key and raw Store identity.

```powershell
cargo run -p pytxo-cli --features experimental-routing-benchmark --bin experimental-routing-trace-export -- --repo C:\path\to\repo --registration REGISTRATION.json --assignment-id ASSIGNMENT_ID --output TRACE.json
```

The exporter canonicalizes the repository, opens its configured existing Store
read-only, requires the binding's domain/run/task, and checks the selected arm
and all five frozen mission pins against the Store projection. It creates
`TRACE.json` once and refuses an existing path. It does not call Jev, run an
agent, change billing, or turn on live routing. A live RJ trace is unavailable
until that separate experimental path is authorized and executed; a shadow
trace cannot stand in for it. Keep `REGISTRATION.json` private because it
contains alias keys and raw run identities. Review `TRACE.json` before wider
sharing despite its allowlisted projection.

For confirmation, a controller-owned execution-order source is also required.
It records one `assignment_started` receipt per assigned run, in the order runs
actually started. Its top-level fields are `schema_version: 1`, the frozen
`study_digest`, an opaque `controller_id`, and `events`. Every event contains
`sequence` (starting at 1), `kind: "assignment_started"`, `assignment_id`, the
frozen `scope_digest` and `task_digest`, and a unique
`controller_receipt_digest`. The recorder checks the **entire** sequence
against `FROZEN.json`'s randomized crossover schedule, so a grouped R0-then-RJ
run order cannot become a valid paired trial. Persist the controller source as
an immutable private artifact, then write its normalized, source-hashed ledger
once:

```powershell
node tooling/benchmarks/routing/record-trial.mjs --record-order STUDY.json CONTROLLER-SOURCE.json ORDER-LEDGER.json
```

The evidence index for confirmation must additionally name
`execution_order_source` and `execution_order_ledger` relative to its private
directory. Assembly re-reads both, checks the source hash and ledger digest,
and refuses a missing, changed, duplicate, or misordered source. Fixture studies
do not require this 1,600-event ledger. Controller provenance, actual execution
order, and the source's immutable storage still need independent audit; a
self-authored JSON file cannot prove them.

After execution, prepare a private evidence directory with one
`EVIDENCE-INDEX.json` row per frozen assignment. Each row names **five separate
JSON files**: `trace`, `acceptance`, `charges`, `timing`, and `authority`.
Paths are relative to the index directory and cannot escape it. The `trace`
must be the Store v3 export made with that assignment's private key; R0 uses
`rules` mode and RJ uses `live` mode. A `shadow` trace cannot claim treatment
benefit. Each other receipt repeats `assignment_id`, `scope_digest`, and
`task_digest` from `STUDY.json` and has source artifacts in the same private
directory:

| Receipt | Required fields and source |
| --- | --- |
| `acceptance` | Frozen `task_contract_digest` and `snapshot_digest`, boolean `accepted`, `outcome_provenance: "independent_blinded"`, opaque `reviewer_id`, separate `oracle_source`/`review_source` paths and SHA-256 digests. |
| `charges` | Assigned billing mode, six cost components and total in nano-USD (or explicit `null`), `usage_reconciled`, exact lists of all trace advisor-request aliases and known/estimated attempt-usage receipt aliases. `provider_charge_provenance` has `kind` (`provider_invoice`, `provider_usage_pinned_rate`, `adapter_report`, `estimate`, or `unavailable`), separate `source` path and `source_sha256`; unavailable uses `null` source and cost. |
| `timing` | `clock_source: "monotonic"`, positive `end_to_end_ms`, nonnegative foreground routing wait and human intervention milliseconds, timing `source` path and SHA-256. |
| `authority` | Boolean `authority_violation`, `ownership_reconciled: true`, controller audit `source` path and SHA-256. |

The receipt source files remain private; the recorder checks that they exist,
stay under the evidence directory, and match their declared hashes. It rejects
missing/duplicate assignments, trace task/scope/arm drift, unlinked decisions,
all five missing/drifted mission pins, reused or wrong-ordinal completed Jev
results, an `applied` label without a completed bound result and first everyday
admission, unsettled ownership, incomplete
source receipts, and cost arithmetic errors.
Unknown charges stay unknown; they cannot become zero-dollar savings. A
provider invoice can account for an uncertain local send, but its allocation
still needs independent review.

```powershell
node tooling/benchmarks/routing/record-trial.mjs --assemble STUDY.json EVIDENCE-INDEX.json BUNDLE.json REVIEW.json
node tooling/benchmarks/routing/analyze-trial.mjs BUNDLE.json REPORT.json
```

All three recorder commands write once and refuse an existing output. `BUNDLE.json` is the
analyzer input. `REVIEW.json` records hashes of every trace, normalized receipt,
external source, controller order ledger, and analysis bundle, without task text
or paths. Keep the study, private evidence directory, bundle, review ledger, and external freeze
registration together for audit. The recorder verifies structural links and
file bytes; it cannot authenticate that a reviewer was truly blinded, that a
provider issued a charge file, that an oracle is sound, that timing is truly
monotonic, that the private registration itself was an independent preregistered
source, that each run used a fresh pinned snapshot, or that the manifest
was archived before outcomes. Independent provider, reviewer, and controller
run-provenance checks remain separate trial controls. It never activates Jev or
authorizes production routing.

The analysis input is one JSON object with `protocol`, `assignments`, and
`observations`. Confirmation additionally requires `frozen_confirmation` (the
complete frozen artifact) and `registered_freeze_digest` (the digest from a
separately archived registration). Each confirmation observation must carry
that same `freeze_digest`; fixture observations have no freeze digest. The
analyzer checks JSON consistency and rejects policy/schedule/assignment drift,
but cannot authenticate a self-asserted registration or execution receipt.
The protocol has `schema_version: 1`, an opaque `campaign_id`, `phase` equal to
`fixture` or `confirmation`, a nonzero uint32 `analysis_seed`, and
`bootstrap_resamples: 10000`. Confirmation is fixed to 400 held-out cases from
200 repositories, two tasks per repository, two repetitions of R0 and RJ per
case, and the six-family 67/67/67/67/66/66 balance. Changing those parameters
requires a new preregistered analysis version before holdout results are read.

Every assignment has an opaque `assignment_id`, `case_id`,
`repository_group_id`, `billing_cohort_id`, `arm` (`R0` or `RJ`), repetition
(1 or 2), `family`, `split` (`fixture` or `holdout`), SHA-256 `snapshot_digest`
and `packet_digest`,
and `billing_mode` (`api`, `subscription`, or `local`). The assignment manifest
digest binds the protocol and sorted assignments; the confirmation freeze digest
also binds pins, case-manifest identity and run order. Each observation refers
to one assignment and both applicable digests. A complete observation has:

| Field | Meaning |
| --- | --- |
| `status: "complete"` | All required outcome and accounting fields are present. `missing` and `incomplete` keep the assigned row visible and suppress all statistical conclusions. |
| `accepted` and `outcome_provenance: "independent_blinded"` | Blinded, independent oracle/reviewer acceptance; worker claims are insufficient. |
| `cost_nano_usd`, `cost_components_nano_usd`, `cost_evidence`, `usage_reconciled` | All-assigned-run cost. The six required component totals are `advisor`, `worker`, `checks`, `handoff`, `rework`, and `differing_infrastructure`; their sum must match the total. Only provider billed or provider usage plus a pinned rate counts toward a conditional dollar ratio. An unknown component and total stay `null`. |
| `end_to_end_ms`, `foreground_routing_wait_ms`, `human_intervention_ms` | Nonnegative measured durations. End-to-end must be positive. A missing or censored measurement stays `null`. |
| `authority_violation` | Independently reconciled result for unauthorized transitions, leaks and unaccounted launches. |
| `routing_exposure` | Recorder-derived count of local may-send and completed journal rows, plus the first admitted route reason, role and advice status (or `null` if no first admission). This is an alias-only projection of Store events, not a provider invoice or complete Core eligibility receipt. |

The upstream recorder/exporter must reconcile each complete observation against
Store events, every charge and uncertain send, provider receipts, the frozen
manifest, actual execution order and fresh snapshots, pinned policies and
profiles, independent oracles and monotonic timing. A separate stress receipt
must cover unauthorized transitions, secret disclosure and unaccounted launches.
This script rejects conflicting/extra IDs and checks component arithmetic, but
it **cannot authenticate** a self-asserted `registered_freeze_digest`,
`usage_reconciled`,
`outcome_provenance` or `authority_violation` value in arbitrary JSON; nor can
it prove that a manifest was archived before outcomes were seen. It therefore
reports numerical `conditional_thresholds` only and always sets
`statistical_gates_passed` and `production_routing_authorized` to `false` in
this scaffold. The v2 report separates frozen planned opportunities from
bundle-reported RJ may-sends, completed sends and first admissions marked
`applied` by Core. It also counts rules-arm strong-default admissions
and applied routes outside the planned stratum. A numerical win with zero
Jev-influenced admissions is explicitly flagged as no Jev efficacy evidence.
Direct analyzer input can fabricate these fields; treat `reported_exposure` as
an unauthenticated bundle claim until the private Store trace and recorder review
ledger are independently checked. These counts also cannot prove full pre-send
Core eligibility. The eventual export must not include prompts, raw packets,
credentials, paths, advisor responses, hidden checks, or worker logs.

The analysis keeps both arms and both repetitions together while resampling
repositories, then paired tasks within each repository, 10,000 times with the
registered seed. It reports one-sided 95% bounds for quality, cost per accepted
task ratio, p95 end-to-end latency ratio and additional intervention time. If
fewer than ten repositories have a discordant paired outcome, quality instead
uses the conservative one-sided Clopper-Pearson harmed-repository bound. Unknown
cost or a non-API/mixed cohort makes the dollar gate inconclusive; zero accepted
runs produces an infinite, failing cost ratio. Missing observations fail closed.
The output includes per-family results and repository-weighted quality and cost
sensitivity values. Conditional numbers alone are not evidence that Jev wins
and never authorize routing.

Run the synthetic contract tests with:

```powershell
node --test tooling/benchmarks/routing/*.test.mjs
```

After an independently validated, frozen bundle exists, analyze it locally:

```powershell
node tooling/benchmarks/routing/analyze-trial.mjs BUNDLE.json REPORT.json
```

The report path must not already exist; the tool never overwrites a prior
analysis. Keep trial inputs and reports private until reviewed for release.
