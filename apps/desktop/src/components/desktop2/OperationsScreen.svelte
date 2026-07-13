<script lang="ts">
  import { IconArrowUpRight, IconCircleCheck, IconShieldCheck } from "@tabler/icons-svelte";
  import type { DesktopSnapshot } from "../../lib/desktop-backend";
  let { snapshot, onRoute }: { snapshot: DesktopSnapshot; onRoute: (route: "approvals" | "runs" | "flow") => void } = $props();
  const activeRuns = $derived(snapshot.runs.filter((run) => ["running", "pending", "dispatching"].includes(run.status)).length);
  const healthyAgents = $derived(snapshot.agents.filter((agent) => agent.status !== "failed").length);
</script>

<section class="screen operations">
  <header class="screen-heading">
    <div><p class="eyebrow">Live supervision</p><h1>Operations</h1><p>One calm view of active work, risk, and system health.</p></div>
    <button class="primary" onclick={() => onRoute("flow")}>New Flow <IconArrowUpRight size={16} /></button>
  </header>

  <div class="metrics" aria-label="Operations summary">
    <article><span>Active runs</span><strong>{activeRuns}</strong><small>Across {snapshot.domains.length} execution domains</small></article>
    <article><span>Agent health</span><strong>{healthyAgents}/{snapshot.agents.length}</strong><small class="good"><IconCircleCheck size={14} /> All responsive</small></article>
    <article><span>Approvals</span><strong>{snapshot.approvals.length}</strong><small>{snapshot.approvals.length ? "Operator review required" : "Inbox clear"}</small></article>
    <article><span>Isolation</span><strong>100%</strong><small class="good"><IconShieldCheck size={14} /> Blast Shield active</small></article>
  </div>

  <div class="workspace-grid">
    <article class="panel run-panel">
      <div class="panel-head"><div><p class="eyebrow">Execution yard</p><h2>Active runs</h2></div><button class="quiet" onclick={() => onRoute("runs")}>View all</button></div>
      <div class="run-list">
        {#each snapshot.runs as run}
          <button class="run-row" onclick={() => onRoute("runs")}>
            <span class:running={run.status === "running"} class="status-dot"></span>
            <span class="run-copy"><strong>{run.repo_root.split(/[\\/]/).pop()}</strong><small>{run.id} · {run.permission_profile ?? "orbit"}</small></span>
            <span class="run-state">{run.status}</span>
            <span class="run-cost">${(run.estimated_cost_usd ?? 0).toFixed(2)}</span>
          </button>
        {/each}
      </div>
    </article>

    <article class="panel approval-panel">
      <div class="panel-head"><div><p class="eyebrow">Decision queue</p><h2>Needs attention</h2></div><button class="quiet" onclick={() => onRoute("approvals")}>Open inbox</button></div>
      {#if snapshot.approvals.length}
        {#each snapshot.approvals as approval}
          <div class="decision-card"><span class="risk">Review</span><h3>{approval.action}</h3><p>{approval.reason}</p><small>{approval.agent_key} · {approval.domain_id}</small></div>
        {/each}
      {:else}
        <div class="empty"><IconCircleCheck size={26} /><strong>Nothing needs review</strong><span>New approval requests will appear here.</span></div>
      {/if}
    </article>
  </div>

  <article class="panel activity-panel">
    <div class="panel-head"><div><p class="eyebrow">Structured events</p><h2>Recent activity</h2></div><span class="live">Live</span></div>
    <div class="timeline">
      <div><time>Now</time><span class="event-mark teal"></span><p><strong>desktop</strong> is implementing the route shell</p><small>run-8f2c · wave 1</small></div>
      <div><time>2m</time><span class="event-mark violet"></span><p>Signal Core enriched 12 structural paths</p><small>context saved · 18.2k tokens</small></div>
      <div><time>6m</time><span class="event-mark gold"></span><p>Blast Shield prepared an isolated review</p><small>14 files · no root writes</small></div>
    </div>
  </article>
</section>
