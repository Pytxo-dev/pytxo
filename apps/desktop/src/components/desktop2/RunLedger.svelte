<script lang="ts">
  import { agentState } from "../../lib/epistemic";
  import type { AgentDto, PermissionEnforcementReceipt, RunReviewDto } from "../../lib/types";
  import AgentInspector from "./AgentInspector.svelte";
  import StateChip from "./StateChip.svelte";

  let {
    agents,
    taskDescriptions = {},
    plan = null,
    agentReceipts = null,
    selectedAgentId = null,
    onSelect,
    inlineInspector = true,
  }: {
    agents: AgentDto[];
    taskDescriptions?: Record<string, string>;
    plan?: RunReviewDto["plan"] | null;
    agentReceipts?: Record<string, PermissionEnforcementReceipt> | null;
    selectedAgentId?: string | null;
    onSelect: (agentId: string) => void;
    inlineInspector?: boolean;
  } = $props();

  let rowButtons: HTMLButtonElement[] = $state([]);

  const ordered = $derived(
    [...agents].sort((a, b) => a.wave - b.wave || a.task_id.localeCompare(b.task_id)),
  );

  /**
   * Claimed paths come from the run's stored plan, not from the agent record.
   * When the plan is unavailable the cell says so rather than showing nothing,
   * which would read as "this task claimed no paths".
   */
  const tasksById = $derived.by(() => {
    const map = new Map<string, RunReviewDto["plan"]["waves"][number][number]>();
    for (const wave of plan?.waves ?? []) {
      for (const task of wave) map.set(task.task_id, task);
    }
    return map;
  });

  const waveBands = $derived.by(() => {
    const bands = new Map<number, AgentDto[]>();
    for (const agent of ordered) {
      const bucket = bands.get(agent.wave);
      if (bucket) bucket.push(agent);
      else bands.set(agent.wave, [agent]);
    }
    return [...bands.entries()].map(([wave, members]) => ({
      wave,
      members,
      reported: members.filter((agent) => {
        const tone = agentState(agent).tone;
        return tone !== "active" && tone !== "queued";
      }).length,
    }));
  });

  function onKeydown(event: KeyboardEvent, index: number) {
    if (event.defaultPrevented || event.metaKey || event.ctrlKey || event.altKey || event.shiftKey) return;
    const key = event.key.toLowerCase();
    const delta = key === "j" || key === "arrowdown" ? 1 : key === "k" || key === "arrowup" ? -1 : 0;
    if (!delta) return;
    event.preventDefault();
    const next = (index + delta + ordered.length) % ordered.length;
    rowButtons[next]?.focus();
    onSelect(ordered[next].id);
  }
</script>

<article class="ledger" aria-labelledby="ledger-title">
  <header>
    <h2 id="ledger-title">What the agents are doing</h2>
  </header>

  {#if ordered.length}
    <div class="head" aria-hidden="true">
      <span>Task</span><span>Agent</span><span>Status</span>
    </div>
    <div class="body">
      {#each waveBands as band (band.wave)}
        <div class="band">
          Step {band.wave + 1}
          <em>{band.reported}/{band.members.length} reported back</em>
        </div>
        {#each band.members as agent (agent.id)}
          {@const state = agentState(agent)}
          {@const index = ordered.findIndex((candidate) => candidate.id === agent.id)}
          {@const task = tasksById.get(agent.task_id)}
          {@const description = taskDescriptions[agent.task_id] ?? agent.task_id}
          {@const receiptId = agent.id.startsWith(`${agent.run_id}:`) ? agent.id.slice(agent.run_id.length + 1) : agent.id}
          <button
            bind:this={rowButtons[index]}
            class="row"
            data-tone={state.tone}
            aria-pressed={selectedAgentId === agent.id}
            aria-label={`${description}, ${state.label}, task ${agent.task_id}, ${agent.id}, exit ${agent.exit_code ?? "not reported"}`}
            class:selected={selectedAgentId === agent.id}
            onclick={() => onSelect(agent.id)}
            onkeydown={(event) => onKeydown(event, index)}
            title={state.detail}
          >
            <span class="edge" aria-hidden="true"></span>
            <span class="task" title={`${description} (${agent.task_id})`}>{description}</span>
            <span class="agent" title={agent.id}>{agent.launcher?.display_name ?? `Agent ${index + 1}`}</span>
            <StateChip tone={state.tone} label={state.label} />
          </button>
          {#if inlineInspector && selectedAgentId === agent.id}
            <AgentInspector {agent} task={task ?? null} receipt={agentReceipts?.[agent.id] ?? agentReceipts?.[receiptId] ?? null} />
          {/if}
        {/each}
      {/each}
    </div>
  {:else}
    <div class="empty">
      <strong>No agents have started yet</strong>
      <span>Their work will appear here when the job starts.</span>
    </div>
  {/if}
</article>

<style>
  .ledger{display:flex;min-width:0;flex-direction:column;overflow:hidden;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-panel-radius,6px);background:var(--pytxo-surface-panel)}
  header{display:flex;align-items:center;justify-content:space-between;gap:16px;min-height:40px;padding:8px 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  header h2{margin:0;font-size:14px;font-weight:600;letter-spacing:0}

  .head,.row{display:grid;grid-template-columns:minmax(90px,1.4fr) minmax(70px,1fr) minmax(100px,1fr);align-items:center;gap:12px;padding:0 14px;text-align:left}
  .head{min-height:30px;border-bottom:1px solid var(--pytxo-line-soft);color:var(--pytxo-text-muted);font:12px var(--pytxo-font-ui);letter-spacing:0}
  .body{display:flex;flex-direction:column;overflow:auto}

  /* Waves are the unit of scheduling, so they are the unit of grouping. The
     count is reported agents over total, which the orchestrator can prove. */
  .band{display:flex;align-items:center;gap:10px;min-height:30px;padding:0 14px;background:var(--pytxo-surface-raised);color:var(--pytxo-text-soft);font:12px var(--pytxo-font-ui);letter-spacing:0}
  .band em{color:var(--pytxo-text-muted);font-style:normal;letter-spacing:0;text-transform:none}

  .row{position:relative;min-height:60px;padding-block:8px;border:0;border-bottom:1px solid var(--pytxo-line-soft);background:transparent;color:inherit;cursor:pointer;font-family:inherit;transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease)}
  .row:last-child{border-bottom:0}
  .row:hover{background:color-mix(in oklab,var(--pytxo-surface-raised) 55%,transparent)}
  .row.selected{background:var(--pytxo-surface-active)}
  .row:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .edge{position:absolute;inset-block:0;left:0;width:2px;background:var(--tone)}

  .task{display:-webkit-box;overflow:hidden;color:var(--pytxo-text-strong);font-size:14px;font-weight:500;line-height:1.45;line-clamp:2;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow-wrap:anywhere}
  .agent{overflow:hidden;color:var(--pytxo-text-muted);font:13px var(--pytxo-font-ui);text-overflow:ellipsis;white-space:nowrap}
  .row :global(.chip strong){white-space:normal;line-height:1.4;overflow:visible;overflow-wrap:anywhere}

  .empty{display:flex;min-height:200px;flex-direction:column;align-items:center;justify-content:center;gap:6px;color:var(--pytxo-text-muted);text-align:center}
  .empty strong{color:var(--pytxo-text-soft);font-size:13px}
  .empty span{max-width:320px;font-size:12px;line-height:1.5}
  @media(max-width:600px){.head,.row{grid-template-columns:minmax(80px,1.2fr) minmax(65px,1fr) minmax(90px,1fr);gap:6px;padding-inline:10px}.ledger{overflow-x:auto}.body,.head{min-width:0}}
</style>
