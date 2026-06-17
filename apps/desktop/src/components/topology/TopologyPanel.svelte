<script lang="ts">
  type Agent = { id: string; task_id: string; wave: number };
  type Arbitrage = {
    agent_id: string;
    saved_tokens: number;
    edited_paths: number;
  };
  type Node = {
    id: string;
    x: number;
    y: number;
    label: string;
    editedPaths: number;
    savedTokens: number;
  };

  let {
    agents = [],
    arbitrage = [],
  }: { agents: Agent[]; arbitrage?: Arbitrage[] } = $props();

  let canvasEl: HTMLCanvasElement | undefined = $state();

  function token(name: string, fallback: string): string {
    if (typeof getComputedStyle === "undefined") return fallback;
    const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
  }

  function layout(nodes: Node[], w: number, h: number): Node[] {
    const cx = w / 2;
    const cy = h / 2;
    const r = Math.min(w, h) / 2 - 28;
    return nodes.map((n, i) => {
      const a = (i / Math.max(nodes.length, 1)) * Math.PI * 2 - Math.PI / 2;
      return { ...n, x: cx + r * Math.cos(a), y: cy + r * Math.sin(a) };
    });
  }

  $effect(() => {
    const canvas = canvasEl;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);

    const teal = token("--brand-teal", "#2dd4bf");
    const violet = token("--brand-violet", "#a78bfa");
    const gold = token("--brand-gold", "#fbbf24");
    const text = token("--foreground", "#e8eaed");

    const byAgent = new Map(arbitrage.map((a) => [a.agent_id, a]));
    const nodes = layout(
      agents.map((a) => {
        const stats = byAgent.get(a.id);
        return {
          id: a.id,
          label: `w${a.wave} ${a.task_id}`,
          x: 0,
          y: 0,
          editedPaths: stats?.edited_paths ?? 0,
          savedTokens: stats?.saved_tokens ?? 0,
        };
      }),
      w,
      h,
    );

    ctx.strokeStyle = violet;
    ctx.lineWidth = 1;
    for (let i = 0; i < nodes.length; i++) {
      const j = (i + 1) % nodes.length;
      if (nodes.length < 2) break;
      ctx.beginPath();
      ctx.moveTo(nodes[i].x, nodes[i].y);
      ctx.lineTo(nodes[j].x, nodes[j].y);
      ctx.stroke();
    }

    for (const n of nodes) {
      const radius = 8 + Math.min(n.editedPaths, 12) * 1.5;
      ctx.fillStyle = n.editedPaths > 0 ? gold : teal;
      ctx.beginPath();
      ctx.arc(n.x, n.y, radius, 0, Math.PI * 2);
      ctx.fill();

      ctx.fillStyle = text;
      ctx.font = "10px system-ui";
      ctx.fillText(n.label, n.x - 28, n.y + radius + 12);
      if (n.editedPaths > 0) {
        ctx.fillStyle = gold;
        ctx.font = "9px system-ui";
        ctx.fillText(
          `${n.editedPaths} paths · ${n.savedTokens} saved`,
          n.x - 28,
          n.y + radius + 23,
        );
      }
    }
  });
</script>

<div class="topology chroma-edge-top">
  <h2 class="panel-title">AST topology (blast radius)</h2>
  <canvas bind:this={canvasEl} width="280" height="220" class="chroma-border"></canvas>
</div>

<style>
  .topology {
    padding-top: 0.75rem;
    margin-top: 0.75rem;
  }
  canvas {
    width: 100%;
    background: var(--void-elevated);
    border-radius: 8px;
  }
</style>
