<script lang="ts">
  import { ipc } from "../../lib/ipc";
  import type { FleetNodeDto, FleetRunDto } from "../../lib/types";

  let {
    runs = [],
  }: {
    runs: FleetRunDto[];
  } = $props();

  let selectedRunId = $state<string | null>(null);
  let nodes = $state<FleetNodeDto[]>([]);
  let selectedNodeId = $state<string | null>(null);
  let canvasEl: HTMLCanvasElement | undefined = $state();

  const waves = $derived(() => {
    const map = new Map<number, FleetNodeDto[]>();
    for (const n of nodes) {
      const list = map.get(n.wave) ?? [];
      list.push(n);
      map.set(n.wave, list);
    }
    return [...map.entries()].sort((a, b) => a[0] - b[0]);
  });

  const selectedNode = $derived(nodes.find((n) => n.node_id === selectedNodeId) ?? null);

  type LayoutNode = FleetNodeDto & { x: number; y: number; r: number };

  function layoutWaveGraph(): LayoutNode[] {
    const waveList = waves();
    const padX = 36;
    const padY = 28;
    const rowGap = 72;
    const colGap = 88;
    const out: LayoutNode[] = [];
    waveList.forEach(([waveIdx, waveNodes], row) => {
      const rowWidth = (waveNodes.length - 1) * colGap;
      const startX = padX + (waveNodes.length === 1 ? 40 : 0);
      waveNodes.forEach((node, col) => {
        out.push({
          ...node,
          x: startX + col * colGap - rowWidth / 2 + 120,
          y: padY + row * rowGap,
          r: 14,
        });
      });
      void waveIdx;
    });
    return out;
  }

  function token(name: string, fallback: string): string {
    if (typeof getComputedStyle === "undefined") return fallback;
    const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
  }

  function drawGraph() {
    const canvas = canvasEl;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const layout = layoutWaveGraph();
    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);

    const violet = token("--brand-violet", "#a78bfa");
    const teal = token("--brand-teal", "#2dd4bf");
    const gold = token("--brand-gold", "#fbbf24");
    const text = token("--foreground", "#e8eaed");
    const bg = token("--void-elevated", "#0a0a12");

    ctx.fillStyle = bg;
    ctx.fillRect(0, 0, w, h);

    const byId = new Map(layout.map((n) => [n.node_id, n]));
    ctx.strokeStyle = violet;
    ctx.lineWidth = 1;
    for (const n of layout) {
      const waveList = waves();
      const waveIdx = waveList.findIndex(([, list]) => list.some((x) => x.node_id === n.node_id));
      if (waveIdx > 0) {
        const prevWave = waveList[waveIdx - 1][1];
        for (const prev of prevWave) {
          const a = byId.get(prev.node_id);
          const b = byId.get(n.node_id);
          if (!a || !b) continue;
          ctx.beginPath();
          ctx.moveTo(a.x, a.y + a.r);
          ctx.lineTo(b.x, b.y - b.r);
          ctx.stroke();
        }
      }
    }

    for (const n of layout) {
      const selected = n.node_id === selectedNodeId;
      ctx.fillStyle =
        n.status === "failed" ? gold : n.status === "completed" ? teal : violet;
      ctx.beginPath();
      ctx.arc(n.x, n.y, selected ? n.r + 3 : n.r, 0, Math.PI * 2);
      ctx.fill();
      if (selected) {
        ctx.strokeStyle = text;
        ctx.lineWidth = 2;
        ctx.stroke();
      }
      ctx.fillStyle = text;
      ctx.font = "10px system-ui";
      ctx.textAlign = "center";
      ctx.fillText(n.node_id, n.x, n.y + n.r + 12);
    }
  }

  function hitTest(ev: MouseEvent): string | null {
    const canvas = canvasEl;
    if (!canvas) return null;
    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;
    const mx = (ev.clientX - rect.left) * scaleX;
    const my = (ev.clientY - rect.top) * scaleY;
    for (const n of layoutWaveGraph()) {
      const dx = mx - n.x;
      const dy = my - n.y;
      if (dx * dx + dy * dy <= (n.r + 4) * (n.r + 4)) return n.node_id;
    }
    return null;
  }

  function onCanvasClick(ev: MouseEvent) {
    const hit = hitTest(ev);
    selectedNodeId = hit === selectedNodeId ? null : hit;
    drawGraph();
  }

  async function selectRun(runId: string) {
    selectedRunId = runId;
    selectedNodeId = null;
    nodes = await ipc.fleetRunStatus(runId);
  }

  $effect(() => {
    nodes;
    selectedNodeId;
    canvasEl;
    drawGraph();
  });
</script>

<h2 class="panel-title">Fleet runs</h2>
{#if runs.length === 0}
  <p class="empty">No fleet runs in catalog.</p>
{:else}
  <ul class="fleet">
    {#each runs as run}
      <li class="fleet-item">
        <button
          class:selected={selectedRunId === run.id}
          onclick={() => selectRun(run.id)}
        >
          <div class="fleet-id">{run.fleet_id}</div>
          <div class="fleet-meta">
            <span class="status status--{run.status}">{run.status}</span>
            <span class="ts">{run.started_at}</span>
          </div>
        </button>
      </li>
    {/each}
  </ul>
{/if}

{#if nodes.length > 0}
  <h3 class="dag-title">Fleet wave graph</h3>
  <canvas
    bind:this={canvasEl}
    width="260"
    height={Math.max(120, waves().length * 72 + 40)}
    class="wave-canvas chroma-border"
    onclick={onCanvasClick}
  ></canvas>
  {#if selectedNode}
    <div class="node-detail">
      <div><strong>{selectedNode.node_id}</strong> · wave {selectedNode.wave}</div>
      <div class="detail-row">domain: {selectedNode.domain_id}</div>
      {#if selectedNode.domain_run_id}
        <div class="detail-row">run: {selectedNode.domain_run_id}</div>
      {/if}
      <div class="detail-row">status: {selectedNode.status}</div>
    </div>
  {/if}
{/if}

<style>
  .empty {
    font-size: 0.8rem;
    opacity: 0.7;
  }
  ul.fleet {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.45rem;
  }
  .fleet-item button {
    width: 100%;
    text-align: left;
    padding: 0.4rem 0.5rem;
    border-radius: 6px;
    background: color-mix(in oklab, var(--foreground) 6%, transparent);
  }
  .fleet-item button.selected {
    border-color: var(--brand-violet);
  }
  .fleet-id {
    font-weight: 600;
    font-size: 0.85rem;
  }
  .fleet-meta {
    display: flex;
    gap: 0.5rem;
    font-size: 0.7rem;
    opacity: 0.8;
  }
  .status--running,
  .status--completed,
  .status--dispatching {
    color: var(--brand-teal);
  }
  .status--failed {
    color: var(--brand-gold);
  }
  .dag-title {
    font-size: 0.8rem;
    color: var(--brand-violet);
    margin: 0.75rem 0 0.35rem;
  }
  .wave-canvas {
    width: 100%;
    cursor: pointer;
    border-radius: 6px;
    background: var(--void-elevated);
  }
  .node-detail {
    margin-top: 0.45rem;
    padding: 0.4rem 0.5rem;
    font-size: 0.72rem;
    border-radius: 6px;
    background: color-mix(in oklab, var(--brand-violet) 10%, transparent);
    font-family: ui-monospace, monospace;
  }
  .detail-row {
    opacity: 0.85;
    margin-top: 0.15rem;
  }
</style>
