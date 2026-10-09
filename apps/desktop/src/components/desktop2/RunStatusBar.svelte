<script lang="ts">
  import { onDestroy } from "svelte";
  import type { AgentDto, RunDto, RunReviewDto } from "../../lib/types";

  // A one-line readout of the focused run, from recorded state only: worker
  // counts, time since the run started while it runs, and where review stands.
  let { run, agents, review }: { run: RunDto; agents: AgentDto[]; review: RunReviewDto | null } = $props();

  const live = (agent: AgentDto) => ["running", "starting", "pending"].includes(agent.status);
  const working = $derived(agents.filter(live).length);
  const done = $derived(agents.filter((agent) => agent.status === "completed" && agent.exit_code === 0).length);
  const failed = $derived(agents.filter((agent) => agent.status === "failed" || (agent.status === "completed" && agent.exit_code !== 0 && agent.exit_code !== null)).length);
  const planned = $derived((review?.run_id === run.id ? review.plan?.waves.flat().length : 0) ?? 0);
  const waiting = $derived(Math.max(0, planned - agents.length));
  const running = $derived(run.status === "running" || working > 0);
  const applyStatus = $derived(((review?.run_id === run.id ? review.apply_status : null) ?? run.apply_status ?? "").toLowerCase());
  const reviewState = $derived(
    applyStatus === "applied" ? { text: "Applied", tone: "done" }
    : applyStatus === "stale" ? { text: "Review is stale", tone: "attention" }
    : applyStatus === "ready" ? { text: "Ready for review", tone: "ready" }
    : applyStatus === "applying" ? { text: "Applying", tone: "live" }
    : applyStatus === "recovery_required" ? { text: "Recovery needed", tone: "failed" }
    : running ? { text: "Review after agents finish", tone: "muted" }
    : { text: "No prepared change yet", tone: "muted" },
  );

  let now = $state(Date.now());
  const timer = setInterval(() => { if (running) now = Date.now(); }, 1000);
  onDestroy(() => clearInterval(timer));
  const started = $derived(Date.parse(run.started_at));
  const elapsed = $derived.by(() => {
    const seconds = Math.max(0, Math.floor((now - started) / 1000));
    const hours = Math.floor(seconds / 3600), minutes = Math.floor((seconds % 3600) / 60), rest = seconds % 60;
    return hours ? `${hours}:${String(minutes).padStart(2, "0")}:${String(rest).padStart(2, "0")}` : `${minutes}:${String(rest).padStart(2, "0")}`;
  });
  const startedAt = $derived(Number.isFinite(started) ? new Date(started).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" }) : null);
</script>

<footer class="run-status" aria-label="Run status">
  <span class="count" data-tone={working ? "live" : "idle"}>{#if working}<span class="tui-spin" aria-hidden="true"><span>.:+*=x</span></span>{:else}<i aria-hidden="true"></i>{/if}<b>{working}</b> working</span>
  <span class="count" data-tone="done"><b>{done}</b> done</span>
  {#if failed}<span class="count" data-tone="failed"><b>{failed}</b> failed</span>{/if}
  {#if waiting}<span class="count"><b>{waiting}</b> waiting</span>{/if}
  <span class="clock">{#if running}<span class="sr">Running for </span>{elapsed}{:else if startedAt}Started {startedAt}{/if}</span>
  <span class="review" data-tone={reviewState.tone}>{reviewState.text}</span>
</footer>

<style>
  /* A terminal status line: mono segments split by hairlines. */
  .run-status { display: flex; flex: 0 0 auto; align-items: center; gap: 14px; min-height: 30px; padding: 0 12px; overflow: hidden; border: 1px solid var(--pytxo-line); border-radius: 6px; background: var(--pytxo-surface-panel); color: var(--pytxo-text-soft); font: 12px var(--pytxo-font-mono); white-space: nowrap; }
  .run-status > span:not(.review) + span:not(.review)::before { content: "│"; margin-right: 14px; color: var(--pytxo-line); }
  .count { display: inline-flex; align-items: center; gap: 6px; }
  .count b { color: var(--pytxo-text-strong); font-weight: 500; font-variant-numeric: tabular-nums; }
  .count i { width: 7px; height: 7px; border-radius: 50%; background: var(--pytxo-text-muted); }
  .count[data-tone="live"], .count[data-tone="live"] b { color: var(--pytxo-activity); }
  .count[data-tone="done"] b { color: var(--state-verified); }
  .count[data-tone="failed"] b { color: var(--state-refuted); }
  .clock { color: var(--pytxo-text-strong); font-weight: 500; font-variant-numeric: tabular-nums; }
  .review { margin-left: auto; padding: 2px 8px; border-radius: 4px; background: var(--pytxo-surface-raised); }
  .review[data-tone="ready"], .review[data-tone="done"] { color: var(--state-verified); }
  .review[data-tone="attention"] { color: var(--state-attention); }
  .review[data-tone="failed"] { color: var(--state-refuted); }
  .review[data-tone="live"] { color: var(--pytxo-activity); }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); }
  @container mission (max-width: 640px) { .clock { display: none; } .run-status { gap: 10px; } }
</style>
