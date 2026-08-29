<script lang="ts">
  import { SURFACE_LABELS, agentState, surfaceTone } from "../../lib/epistemic";
  import type { AgentDto, PermissionEnforcementReceipt, RunReviewDto } from "../../lib/types";
  import StateChip from "./StateChip.svelte";

  let {
    agent,
    task = null,
    receipt = null,
  }: {
    agent: AgentDto;
    task?: RunReviewDto["plan"]["waves"][number][number] | null;
    /**
     * Per-agent enforcement, which the orchestrator records separately from the
     * run-level receipt. A missing entry means this agent's enforcement was not
     * recorded, which is shown rather than inherited from the run.
     */
    receipt?: PermissionEnforcementReceipt | null;
  } = $props();

  const SURFACES = [
    { key: "workspace_isolation", label: "Workspace isolation" },
    { key: "host_filesystem_boundary", label: "Host filesystem" },
    { key: "network", label: "Network" },
    { key: "apply_boundary", label: "Apply boundary" },
  ] as const;

  const state = $derived(agentState(agent));
</script>

<div class="inspector">
  <div class="head">
    <div>
      <p>Agent detail</p>
      <strong>{agent.id}</strong>
    </div>
    <StateChip tone={state.tone} label={state.label} detail={state.detail} layout="stacked" />
  </div>

  <dl class="facts">
    <div><dt>Task</dt><dd>{agent.task_id}</dd></div>
    <div><dt>Wave</dt><dd>{agent.wave + 1}</dd></div>
    <div><dt>Exit code</dt><dd>{agent.exit_code ?? "Not reported"}</dd></div>
    <div><dt>Isolation root</dt><dd>{agent.root_id ?? "Shared working tree"}</dd></div>
    <div class="wide"><dt>Claimed paths</dt><dd>{task ? (task.paths.length ? task.paths.join(", ") : "None declared") : "Plan not loaded"}</dd></div>
    <div class="wide"><dt>Verify</dt><dd>{task ? (task.verify.length ? task.verify.join(" && ") : "No verify command") : "Plan not loaded"}</dd></div>
  </dl>

  <div class="receipt">
    <p>Enforcement for this agent</p>
    {#if receipt}
      {#each SURFACES as surface (surface.key)}
        {@const value = receipt[surface.key]}
        <div class="surface" data-tone={surfaceTone(value.status)}>
          <span>{surface.label}</span>
          <StateChip tone={surfaceTone(value.status)} label={SURFACE_LABELS[value.status]} />
          <small>{value.mechanism}</small>
        </div>
      {/each}
    {:else}
      <div class="surface" data-tone="unknown">
        <span>Not recorded</span>
        <StateChip tone="unknown" label="No agent receipt" />
        <small>The run-level receipt does not carry an entry for this agent.</small>
      </div>
    {/if}
  </div>
</div>

<style>
  /* Opens under the selected ledger row so per-agent evidence is one click from
     the row, without moving the operator off the run. */
  .inspector{display:flex;flex-direction:column;gap:0;border-bottom:1px solid var(--pytxo-line-soft);background:var(--pytxo-surface-raised)}
  .head{display:flex;align-items:flex-start;justify-content:space-between;gap:14px;padding:11px 14px}
  .head p{margin:0 0 3px;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.06em}
  .head strong{font:12px "IBM Plex Mono",monospace}

  .facts{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:1px;margin:0 14px;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:var(--pytxo-line-soft);overflow:hidden}
  .facts>div{padding:7px 9px;background:var(--pytxo-surface-input)}
  .facts .wide{grid-column:1/-1}
  dt{color:var(--pytxo-text-muted);font-size:11px}
  dd{overflow:hidden;margin:3px 0 0;color:var(--pytxo-text-soft);font:11px "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap}

  .receipt{padding:12px 14px 14px}
  .receipt p{margin:0 0 8px;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.06em}
  .surface{display:grid;grid-template-columns:minmax(0,140px) auto minmax(0,1fr);gap:10px;align-items:center;padding:7px 0;border-bottom:1px solid var(--pytxo-line-soft)}
  .surface:last-child{border-bottom:0}
  .surface span{color:var(--pytxo-text-soft);font-size:12px}
  .surface small{overflow:hidden;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap}
  @media(max-width:820px){.surface{grid-template-columns:minmax(0,1fr);gap:4px}}
</style>
