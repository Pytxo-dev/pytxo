<script lang="ts">
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { RoutingDisplayAttempt, RoutingDisplaySummary, RoutingDisplayTask, RunDto } from "../../lib/types";

  let { run, backend, record, onRetry }: {
    run: RunDto; backend: DesktopBackend;
    record?: { summary: RoutingDisplaySummary | null; error: string | null; loading: boolean };
    onRetry?: () => void;
  } = $props();

  let summary = $state<RoutingDisplaySummary | null>(null);
  let error = $state<string | null>(null);
  let loading = $state(false);
  let retryCount = $state(0);
  const activeSummary = $derived(record ? record.summary : summary);
  const activeError = $derived(record ? record.error : error);
  const activeLoading = $derived(record ? record.loading : loading);
  const visibleSummary = $derived(
    activeSummary?.run_id === run.id && activeSummary.domain_id === run.domain_id && activeSummary.routing_revision === run.routing_revision
      ? activeSummary
      : null,
  );

  function label(value: string): string {
    const words = value.replaceAll("_", " ");
    return words.charAt(0).toUpperCase() + words.slice(1);
  }

  function decisionLabel(selection: NonNullable<RoutingDisplaySummary["tasks"][number]["last_decision"]>["selection"]): string {
    if (selection.kind === "blocked") return `Blocked · ${label(selection.code)}`;
    return `${selection.kind === "wait_for_capacity" ? "Waiting for capacity" : "Selected"} · ${label(selection.role)}`;
  }

  function predecessorLabel(task: RoutingDisplayTask, attempt: RoutingDisplayAttempt): string | null {
    if (!attempt.predecessor_id) return null;
    const predecessor = task.attempts.find((candidate) => candidate.attempt_id === attempt.predecessor_id);
    return predecessor
      ? `Follows attempt ${predecessor.ordinal} (${predecessor.attempt_id})`
      : `Follows ${attempt.predecessor_id} (predecessor not in this summary)`;
  }

  function admittedAfterEarlierDecision(task: RoutingDisplayTask): boolean {
    const earlier = task.pre_admission;
    return earlier != null && task.attempts.some((attempt) => attempt.ordinal === earlier.ordinal);
  }

  const summaryHighlights = $derived.by((): string[] => {
    if (!visibleSummary) return [];
    const tasks = visibleSummary.tasks;
    const highlights: string[] = [];
    const recovering = tasks.find((task) => task.state === "recovery_required" || task.attempts.some((attempt) => attempt.state === "recovery_required"));
    if (recovering) highlights.push(`Needs recovery: ${recovering.task_id}`);

    if (visibleSummary.cancelled) {
      const unsettled = tasks.some((task) => task.attempts.some((attempt) =>
        !["passed", "failed", "failed_no_launch", "cancelled", "recovery_required"].includes(attempt.state)));
      highlights.push(`Cancellation recorded${unsettled ? " · attempt settlement may still be pending" : ""}`);
      return highlights.slice(0, 2);
    }

    const capacity = tasks.filter((task) => task.state === "ready"
      && task.pre_admission?.outcome.kind === "not_admitted"
      && task.pre_admission.outcome.stage === "capacity_unavailable"
      && !admittedAfterEarlierDecision(task));
    if (capacity.length) {
      const task = capacity[0];
      const selection = task.pre_admission!.decision.selection;
      const role = selection.kind === "blocked" ? "Route" : label(selection.role);
      highlights.push(`Capacity: ${task.task_id} · ${role} selected · no attempt admitted${capacity.length > 1 ? ` · ${capacity.length - 1} more` : ""}`);
    }

    const current = tasks.flatMap((task) => {
      const attempt = task.attempts.find((candidate) => candidate.attempt_id === task.current_attempt_id);
      return task.state === "active" && attempt && !["passed", "failed", "failed_no_launch", "cancelled", "recovery_required"].includes(attempt.state)
        ? [{ task, attempt }]
        : [];
    });
    if (current.length) {
      const { task, attempt } = current[0];
      highlights.push(`Unsettled attempt: ${task.task_id} · attempt ${attempt.ordinal} · ${label(attempt.role)} via ${attempt.harness_id} · ${label(attempt.state)}${current.length > 1 ? ` · ${current.length - 1} more` : ""}`);
    }
    return highlights.slice(0, 2);
  });

  $effect(() => {
    void retryCount;
    if (record) return;
    const runId = run.id;
    const domainId = run.domain_id;
    const revision = run.routing_revision;
    summary = null;
    error = null;
    if (revision == null) {
      loading = false;
      return;
    }
    let current = true;
    loading = true;
    backend.routingRunSummary(runId, domainId)
      .then((record) => {
        if (!current) return;
        if (!record || record.run_id !== runId || record.domain_id !== domainId || record.routing_revision !== revision) {
          error = "Routing record changed or could not be matched to this run. Refreshing the run list may resolve it.";
          return;
        }
        summary = record;
      })
      .catch((cause: unknown) => {
        if (current) error = cause instanceof Error ? cause.message : String(cause);
      })
      .finally(() => {
        if (current) loading = false;
      });
    return () => { current = false; };
  });
</script>

{#if run.routing_revision != null}
  <!-- The bounded attempt history is a keyboard-scrollable region in Work. -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
  <section class="routing-record" aria-label="Routing and worker attempts" tabindex="0">
    <details>
      <summary><strong>Routing and worker attempts</strong>{#if visibleSummary}<span>{visibleSummary.tasks.length} tasks · {visibleSummary.tasks.reduce((count, task) => count + task.attempts.length, 0)} attempts · {label(visibleSummary.mode)} mode · Recorded routing state</span>{#if summaryHighlights.length}<span class="summary-highlights">{#each summaryHighlights as highlight (highlight)}<span>{highlight}</span>{/each}</span>{/if}{:else if activeLoading}<span>Loading…</span>{:else if activeError}<span>Unavailable</span>{/if}</summary>
      <div class="routing-body">
        <p class="description">Recorded execution choices and receipt presence. A receipt alone does not prove a passed check; repository changes still require Review and Apply.</p>
        {#if activeLoading}<p class="state" role="status">Loading routing record…</p>{/if}
        {#if activeError}<p class="state error" role="alert">Routing record unavailable: {activeError} <button class="retry" onclick={() => onRetry ? onRetry() : retryCount++}>Retry routing record</button></p>{/if}
        {#if visibleSummary}
          {#if visibleSummary.cancelled}<p class="state">Cancellation recorded for this routing mission.</p>{/if}
          <ol class="tasks">
            {#each visibleSummary.tasks as task (task.task_id)}
              <li class="task">
                <div class="task-heading"><strong>{task.task_id}</strong><span>{label(task.state)}</span></div>
                {#each task.dependency_task_ids as dependencyId (dependencyId)}
                  {@const dependency = visibleSummary.tasks.find((candidate) => candidate.task_id === dependencyId)}
                  <p class="meta">After {dependencyId} · {dependency ? label(dependency.state) : "Unknown"}{#if dependency?.winning_attempt_id}{" "}· winning attempt recorded{/if}</p>
                {/each}
                {#if task.last_decision}<p class="decision">Last admitted or blocked decision: {decisionLabel(task.last_decision.selection)} · {label(task.last_decision.advice_status)} · {label(task.last_decision.reason)}</p>{/if}
                {#if task.pre_admission}
                  {@const laterAdmitted = admittedAfterEarlierDecision(task)}
                  <p class="decision">{laterAdmitted ? "Earlier pre-admission decision" : "Pre-admission decision"} for attempt {task.pre_admission.ordinal}: {decisionLabel(task.pre_admission.decision.selection)}{#if task.pre_admission.decision.advice_status !== "not_used"} · {label(task.pre_admission.decision.advice_status)}{/if} · {task.pre_admission.outcome.kind === "pending" ? "Admission outcome not recorded; no attempt recorded for this decision" : laterAdmitted ? `Not admitted · ${label(task.pre_admission.outcome.stage)}; A later decision admitted attempt ${task.pre_admission.ordinal}` : `Not admitted · ${label(task.pre_admission.outcome.stage)}; no attempt admitted for this decision`}</p>
                {/if}
                {#if task.attempts.length}
                  <ol class="attempts" aria-label={`Worker attempts for ${task.task_id}`}>
                    {#each task.attempts as attempt (attempt.attempt_id)}
                      {@const predecessor = predecessorLabel(task, attempt)}
                      <li class="attempt">
                        <div class="attempt-heading"><strong>Worker attempt {attempt.ordinal}</strong><span>{label(attempt.state)}</span>{#if task.winning_attempt_id === attempt.attempt_id}<span class="winner">Winner recorded</span>{/if}</div>
                        <p>{label(attempt.role)} · {attempt.profile_id} · {attempt.harness_id} · {label(attempt.billing_mode)} billing</p>
                        <p class="meta">Route: {decisionLabel(attempt.decision.selection)} · {label(attempt.decision.advice_status)} · {label(attempt.decision.reason)}</p>
                        <p class="meta">Attempt {attempt.attempt_id} · Agent {attempt.agent_id}</p>
                        {#if predecessor}<p class="meta">{predecessor}</p>{/if}
                        <p class="meta">{attempt.handoff_referenced ? "Handoff reference recorded" : "No handoff reference"} · {attempt.sealed_output_recorded ? "Sealed output recorded" : "No sealed output"} · {attempt.checks_receipt_recorded ? "Checker receipt recorded" : "No checker receipt"} · {attempt.ownership_released ? "Ownership released" : "Ownership not released"} · Usage {label(attempt.usage_status).toLowerCase()}</p>
                      </li>
                    {/each}
                  </ol>
                {:else if !task.pre_admission}<p class="meta">No worker attempt recorded.</p>{/if}
              </li>
            {/each}
          </ol>
        {/if}
      </div>
    </details>
  </section>
{/if}

<style>
  .routing-record{min-width:0;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-panel-radius,8px);background:var(--pytxo-surface-panel);color:var(--pytxo-text-strong)}
  .routing-record:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  summary{display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:6px 16px;padding:12px 16px;list-style:none;cursor:pointer}
  summary::-webkit-details-marker{display:none}
  summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  summary strong{font-size:12px;font-weight:650}
  summary span{color:var(--pytxo-text-soft);font-size:11px}
  .summary-highlights{flex-basis:100%;display:flex;flex-wrap:wrap;gap:4px 14px}
  .summary-highlights span{overflow-wrap:anywhere}
  .routing-body{padding:0 16px 16px}
  .description{margin:0;color:var(--pytxo-text-soft);font-size:12px;line-height:1.45}
  .state{margin:12px 0 0;color:var(--pytxo-text-soft);font-size:12px}
  .state.error{color:var(--pytxo-danger, var(--pytxo-text-strong))}
  .retry{margin-left:8px;padding:3px 7px;border:1px solid currentColor;border-radius:4px;background:transparent;color:inherit;font:inherit;cursor:pointer}.retry:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
  .tasks,.attempts{list-style:none;margin:12px 0 0;padding:0}
  .tasks{display:grid;gap:10px}
  .task{min-width:0;padding:11px;border:1px solid var(--pytxo-line-soft);border-radius:6px}
  .task-heading,.attempt-heading{display:flex;flex-wrap:wrap;align-items:baseline;gap:8px}
  .task-heading strong,.attempt-heading strong{min-width:0;overflow-wrap:anywhere;font-size:12px}
  .task-heading span,.attempt-heading span{color:var(--pytxo-text-soft);font-size:11px}
  .attempt-heading .winner{color:var(--pytxo-accent)}
  .decision,.meta,.attempt p{margin:6px 0 0;color:var(--pytxo-text-soft);font-size:11px;line-height:1.45;overflow-wrap:anywhere}
  .attempts{display:grid;gap:7px;margin-top:10px}
  .attempt{min-width:0;padding:9px 10px;border-left:2px solid var(--pytxo-line);background:var(--pytxo-surface-shell)}
</style>
