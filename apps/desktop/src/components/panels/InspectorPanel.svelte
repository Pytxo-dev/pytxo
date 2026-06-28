<script lang="ts">
  import type { HitlDto, StructuralGraphDto } from "../../lib/types";

  let {
    diffText = "",
    dryRunOut = "",
    hitl = [],
    selectedNodeId = null as string | null,
    structural = null as StructuralGraphDto | null,
    isolationBackend = "",
    onLoad,
    onCommit,
    onRespondHitl,
  }: {
    diffText?: string;
    dryRunOut?: string;
    hitl?: HitlDto[];
    selectedNodeId?: string | null;
    structural?: StructuralGraphDto | null;
    isolationBackend?: string;
    onLoad: () => void;
    onCommit: () => void;
    onRespondHitl: (id: string, approve: boolean) => void;
  } = $props();

  const selectedNode = $derived(
    structural?.nodes.find((n) => n.id === selectedNodeId) ?? null,
  );
</script>

<aside class="inspector glass-panel">
  {#if selectedNode}
    <section class="block">
      <h2 class="panel-title">Node</h2>
      <p class="node-id">{selectedNode.id}</p>
      <p class="node-meta">
        {#if selectedNode.edited}
          <span class="tag tag--gold">Edited</span>
        {:else}
          <span class="tag">File</span>
        {/if}
        {#if selectedNode.label.includes("::")}
          <span class="tag tag--violet">Symbol</span>
        {/if}
      </p>
    </section>
  {/if}

  {#if hitl.length > 0}
    <section class="block">
      <h2 class="panel-title panel-title--gold">Approvals ({hitl.length})</h2>
      <ul class="hitl">
        {#each hitl as req}
          <li class="hitl-item">
            <div class="hitl-action">{req.action}</div>
            <div class="hitl-reason">{req.reason}</div>
            <div class="hitl-buttons">
              <button type="button" class="gold" onclick={() => onRespondHitl(req.id, true)}>
                Approve
              </button>
              <button type="button" onclick={() => onRespondHitl(req.id, false)}>Deny</button>
            </div>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <section class="block block--grow">
    <h2 class="panel-title">
      Diff
      {#if isolationBackend}
        <span class="backend-badge">{isolationBackend}</span>
      {/if}
    </h2>
    <div class="diff-actions">
      <button type="button" onclick={onLoad}>Load</button>
      <button type="button" class="gold chroma-glow" onclick={onCommit}>Approve merge</button>
    </div>
    <pre class="diff-body">{diffText || dryRunOut || "Dry-run output appears here after Dry run."}</pre>
  </section>
</aside>

<style>
  .inspector {
    width: 300px;
    flex-shrink: 0;
    margin: 0.5rem 0.5rem 0.5rem 0;
    padding: 0.85rem;
    border-radius: 16px;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-height: 0;
    overflow: auto;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
  }
  .block--grow {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .panel-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
  }
  .node-id {
    font-family: ui-monospace, monospace;
    font-size: 0.75rem;
    word-break: break-all;
    margin: 0 0 0.35rem;
  }
  .node-meta {
    display: flex;
    gap: 0.35rem;
    margin: 0;
  }
  .tag {
    font-size: 0.65rem;
    padding: 0.1rem 0.35rem;
    border-radius: 6px;
    background: color-mix(in oklab, var(--brand-teal) 14%, transparent);
    color: var(--brand-teal);
  }
  .tag--gold {
    background: color-mix(in oklab, var(--brand-gold) 14%, transparent);
    color: var(--brand-gold);
  }
  .tag--violet {
    background: color-mix(in oklab, var(--brand-violet) 14%, transparent);
    color: var(--brand-violet);
  }
  ul.hitl {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .hitl-item {
    border-radius: 10px;
    padding: 0.5rem;
    margin-bottom: 0.4rem;
    background: color-mix(in oklab, var(--brand-gold) 8%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--brand-gold) 25%, transparent);
  }
  .hitl-action {
    font-weight: 600;
    color: var(--brand-gold);
    font-size: 0.85rem;
  }
  .hitl-reason {
    font-size: 0.75rem;
    opacity: 0.85;
    margin: 0.25rem 0 0.4rem;
  }
  .hitl-buttons {
    display: flex;
    gap: 0.35rem;
  }
  .hitl-buttons button {
    flex: 1;
    min-height: 36px;
    border-radius: 8px;
  }
  .backend-badge {
    font-size: 0.65rem;
    padding: 0.1rem 0.35rem;
    border-radius: 6px;
    color: var(--brand-teal);
    background: color-mix(in oklab, var(--brand-teal) 12%, transparent);
  }
  .diff-actions {
    display: flex;
    gap: 0.35rem;
    margin-bottom: 0.5rem;
  }
  .diff-actions button {
    min-height: 36px;
    border-radius: 8px;
  }
  .diff-body {
    flex: 1;
    overflow: auto;
    font-size: 11px;
    margin: 0;
    padding: 0.65rem;
    border-radius: 10px;
    background: color-mix(in oklab, var(--card) 55%, transparent);
    font-family: ui-monospace, monospace;
    box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--foreground) 6%, transparent);
  }
</style>
