<script lang="ts">
  import type { HitlDto, StructuralGraphDto } from "../../lib/types";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { ScrollArea } from "$lib/components/ui/scroll-area";

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

  const isEmpty = $derived(
    !selectedNode && hitl.length === 0 && !dryRunOut && !diffText,
  );
</script>

<aside class="inspector">
  {#if isEmpty}
    <section class="block empty-state">
      <h2 class="panel-title">Inspector</h2>
      <p class="empty-copy">
        Select a file in the topology, or run agents to see diffs and approvals here.
      </p>
    </section>
  {/if}

  {#if selectedNode}
    <section class="block">
      <h2 class="panel-title">Node</h2>
      <p class="node-id">{selectedNode.id}</p>
      <p class="node-meta">
        {#if selectedNode.edited}
          <Badge variant="secondary">Edited</Badge>
        {:else}
          <Badge variant="muted">File</Badge>
        {/if}
        {#if selectedNode.label.includes("::")}
          <Badge variant="outline">Symbol</Badge>
        {/if}
      </p>
    </section>
  {/if}

  {#if hitl.length > 0}
    <section class="block">
      <h2 class="panel-title">Approvals ({hitl.length})</h2>
      <ul class="hitl">
        {#each hitl as req}
          <li class="hitl-item">
            <div class="hitl-action">{req.action}</div>
            <div class="hitl-reason">{req.reason}</div>
            <div class="hitl-buttons">
              <Button size="sm" class="flex-1" onclick={() => onRespondHitl(req.id, true)}>
                Approve
              </Button>
              <Button
                size="sm"
                variant="outline"
                class="flex-1"
                onclick={() => onRespondHitl(req.id, false)}
              >
                Deny
              </Button>
            </div>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if dryRunOut}
    <section class="block">
      <h2 class="panel-title">Preview</h2>
      <ScrollArea class="preview-scroll">
        <pre class="diff-body">{dryRunOut}</pre>
      </ScrollArea>
    </section>
  {/if}

  <section class="block block--grow">
    <div class="diff-header">
      <h2 class="panel-title">
        Diff
        {#if isolationBackend}
          <Badge variant="muted">{isolationBackend}</Badge>
        {/if}
      </h2>
      <div class="diff-actions">
        <Button size="sm" variant="outline" onclick={onLoad}>Load</Button>
        <Button size="sm" onclick={onCommit}>Apply run</Button>
      </div>
    </div>
    <ScrollArea class="diff-scroll">
      <pre class="diff-body">
        {diffText || "Load a diff after agents edit files. Approve merge when you are ready."}
      </pre>
    </ScrollArea>
  </section>
</aside>

<style>
  .inspector {
    width: 300px;
    flex-shrink: 0;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    min-height: 0;
    overflow: hidden;
    border-left: 1px solid var(--border);
    background: var(--card);
  }
  .block {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    min-height: 0;
  }
  .block--grow {
    flex: 1;
    min-height: 0;
  }
  .empty-state {
    padding: 0.25rem 0;
  }
  .empty-copy {
    margin: 0;
    font-size: var(--text-sm, 0.875rem);
    color: var(--muted-foreground);
    line-height: 1.45;
  }
  .panel-title {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin: 0;
    font-size: var(--text-sm, 0.875rem);
    font-weight: 600;
  }
  .node-id {
    font-family: ui-monospace, monospace;
    font-size: var(--text-xs, 0.75rem);
    word-break: break-all;
    margin: 0;
  }
  .node-meta {
    display: flex;
    gap: 0.35rem;
    margin: 0;
    flex-wrap: wrap;
  }
  ul.hitl {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .hitl-item {
    border-radius: var(--panel-radius);
    padding: 0.5rem;
    margin-bottom: 0.4rem;
    border: 1px solid var(--border);
    background: color-mix(in oklab, var(--foreground) 3%, transparent);
  }
  .hitl-action {
    font-weight: 600;
    font-size: var(--text-sm, 0.875rem);
  }
  .hitl-reason {
    font-size: var(--text-xs, 0.75rem);
    color: var(--muted-foreground);
    margin: 0.25rem 0 0.4rem;
  }
  .hitl-buttons {
    display: flex;
    gap: 0.35rem;
  }
  .diff-header {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex-shrink: 0;
  }
  .diff-actions {
    display: flex;
    gap: 0.35rem;
  }
  :global(.preview-scroll) {
    max-height: 140px;
  }
  :global(.diff-scroll) {
    flex: 1;
    min-height: 0;
  }
  .diff-body {
    margin: 0;
    padding: 0.65rem;
    border-radius: var(--panel-radius);
    background: color-mix(in oklab, var(--foreground) 4%, transparent);
    font-family: ui-monospace, monospace;
    font-size: var(--text-xs, 0.75rem);
    line-height: 1.45;
    white-space: pre-wrap;
    word-break: break-word;
    border: 1px solid var(--border);
  }
</style>
