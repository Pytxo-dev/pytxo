<script lang="ts" module>
  export type Blip = { id: string; mark: string; tone: "live" | "done" | "failed" | "queued" | "settled"; ring: number };
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { uiPrefs } from "../../lib/ui-prefs.svelte";
  let { blips, rings }: { blips: Blip[]; rings: number } = $props();

  // An ASCII scope of the fleet: the Pytxo aperture at the core, one orbit per
  // step, one numbered blip per task. Live workers travel their orbit and the
  // sweep turns only while something is live; settled workers stop where they
  // finished. Decoration over real state; the rows beside it carry the text.
  const W = 440, H = 204, CX = W / 2, CY = H / 2;
  let canvas: HTMLCanvasElement;
  let visible = $state(false);
  let hidden = $state(false);
  let reduced = $state(false);
  let ready = $state(false);
  let themeRevision = $state(0);
  const live = $derived(blips.some((blip) => blip.tone === "live"));
  const still = $derived(reduced || uiPrefs.reducedMotion);

  const core = Array.from({ length: 320 }, (_, n) => {
    const y = 1 - 2 * (n + .5) / 320, radius = Math.sqrt(1 - y * y), angle = n * Math.PI * (3 - Math.sqrt(5));
    return { x: Math.cos(angle) * radius, y, z: Math.sin(angle) * radius };
  });
  const angles = new Map<string, number>();
  let phase = 0, sweep = -Math.PI / 2;
  // The loop reads the newest blips without restarting on every poll.
  let latest: Blip[] = [];
  $effect.pre(() => { latest = blips; });

  function ringRadius(ring: number) {
    const count = Math.max(1, rings);
    const rx = count === 1 ? 136 : 86 + (ring / (count - 1)) * (W / 2 - 26 - 86);
    return { rx, ry: rx * .44 };
  }

  onMount(() => {
    const media = matchMedia("(prefers-reduced-motion: reduce)");
    const sync = () => { hidden = document.hidden; reduced = media.matches; };
    sync();
    document.addEventListener("visibilitychange", sync);
    media.addEventListener("change", sync);
    const observer = new IntersectionObserver(([entry]) => { visible = entry.isIntersecting; });
    observer.observe(canvas);
    const themeObserver = new MutationObserver(() => { themeRevision += 1; });
    themeObserver.observe(document.documentElement, { attributes: true, attributeFilter: ["data-chroma-theme"] });
    let alive = true;
    document.fonts.ready.then(() => { if (alive) ready = true; });
    return () => { alive = false; observer.disconnect(); themeObserver.disconnect(); document.removeEventListener("visibilitychange", sync); media.removeEventListener("change", sync); };
  });

  $effect(() => {
    void themeRevision;
    const context = canvas?.getContext("2d");
    if (!context || !ready) return;
    const animate = visible && !hidden && !still;
    // A still scope redraws when the fleet changes; a moving one picks it up next frame.
    if (!animate) void blips;
    const style = getComputedStyle(canvas);
    const token = (name: string, fallback: string) => style.getPropertyValue(name).trim() || fallback;
    const palette = {
      line: token("--pytxo-text-muted", "#8b8f98"),
      live: token("--pytxo-activity", "#45dccb"),
      done: token("--state-verified", "#5fd08a"),
      failed: token("--state-refuted", "#f07a7a"),
      queued: token("--pytxo-text-soft", "#b6bac2"),
      glyph: ["--pytxo-glyph-cyan", "--pytxo-glyph-periwinkle", "--pytxo-glyph-violet", "--pytxo-glyph-warm"].map((name) => token(name, "#b4b9ff")),
    };
    const ratio = window.devicePixelRatio || 1;
    canvas.width = W * ratio;
    canvas.height = H * ratio;
    context.setTransform(ratio, 0, 0, ratio, 0, 0);
    context.font = '500 12px "IBM Plex Mono", Consolas, monospace';
    context.textBaseline = "middle";
    context.textAlign = "center";
    const cw = context.measureText("M").width || 7.2, ch = 13;
    const cols = Math.floor(W / cw), rows = Math.floor(H / ch);
    let frame = 0, previous = 0, disposed = false;

    function draw(time: number) {
      if (disposed || !context) return;
      const current = latest;
      const fleetLive = current.some((blip) => blip.tone === "live");
      const step = previous ? Math.min(time - previous, 100) / 1000 : 0;
      if (animate && previous && time - previous < 1000 / (fleetLive ? 24 : 10)) { frame = requestAnimationFrame(draw); return; }
      previous = time;
      phase += step * (fleetLive ? .7 : .15);
      if (fleetLive) sweep += step * 1.6;
      const cells = new Map<number, { char: string; color: string; alpha: number }>();
      const put = (x: number, y: number, char: string, color: string, alpha: number, force = false) => {
        const col = Math.round(x / cw), row = Math.round(y / ch);
        if (col < 0 || row < 0 || col >= cols || row >= rows) return;
        const key = row * cols + col;
        const existing = cells.get(key);
        if (existing && !force && existing.alpha >= alpha) return;
        cells.set(key, { char, color, alpha });
      };
      for (let ring = 0; ring < Math.max(1, rings); ring += 1) {
        const { rx, ry } = ringRadius(ring);
        for (let a = 0; a < Math.PI * 2; a += cw / rx / 1.4) put(CX + Math.cos(a) * rx, CY + Math.sin(a) * ry, "·", palette.line, .42);
        // Each orbit is a step: its number sits on the left edge of the ring.
        const label = String(ring + 1).padStart(2, "0");
        [...label].forEach((char, index) => put(CX - rx + (index - .5) * cw, CY, char, palette.line, .75, true));
      }
      if (fleetLive) {
        const { rx, ry } = ringRadius(Math.max(0, rings - 1));
        for (let k = 0; k < 16; k += 1) {
          const a = sweep - k * .05;
          for (let s = .3; s <= 1.04; s += .06) put(CX + Math.cos(a) * rx * s, CY + Math.sin(a) * ry * s, k === 0 ? ":" : ".", palette.live, .55 * (1 - k / 16));
        }
      }
      for (const p of core) {
        const x = p.x * Math.cos(phase) + p.z * Math.sin(phase);
        const z = p.z * Math.cos(phase) - p.x * Math.sin(phase);
        if (z < -.1 || Math.abs(p.y - .46 * x) < .17) continue;
        const color = p.y < -.35 ? 0 : p.y < .25 ? 1 : x > .45 ? 3 : 2;
        put(CX + x * 50, CY + p.y * 40, ".:+*=x"[Math.min(5, Math.floor((z + .1) * 5.4))], palette.glyph[color], .5 + .5 * Math.max(0, z), true);
      }
      const perRing = new Map<number, number>();
      for (const blip of current) perRing.set(blip.ring, (perRing.get(blip.ring) ?? 0) + 1);
      const seen = new Map<number, number>();
      for (const blip of current) {
        const index = seen.get(blip.ring) ?? 0;
        seen.set(blip.ring, index + 1);
        const home = (Math.PI * 2 * index) / (perRing.get(blip.ring) ?? 1) - Math.PI / 2 + blip.ring * .9;
        let angle = angles.get(blip.id) ?? home;
        if (blip.tone === "live") angle += step * (.55 - blip.ring * .1);
        angles.set(blip.id, angle);
        const { rx, ry } = ringRadius(blip.ring);
        const at = (a: number) => [CX + Math.cos(a) * rx, CY + Math.sin(a) * ry] as const;
        const color = palette[blip.tone === "settled" ? "queued" : blip.tone];
        if (blip.tone === "live") ["•", "·", ".", "."].forEach((char, k) => { const [x, y] = at(angle - (k + 1) * .14); put(x, y, char, color, .7 - k * .15, true); });
        const near = fleetLive && Math.abs(((sweep - angle) % (Math.PI * 2) + Math.PI * 2) % (Math.PI * 2)) < .5;
        const [x, y] = at(angle);
        put(x - cw, y, "[", color, blip.tone === "queued" ? .45 : .8, true);
        put(x, y, blip.mark, color, blip.tone === "queued" ? .55 : near ? 1 : .92, true);
        put(x + cw, y, "]", color, blip.tone === "queued" ? .45 : .8, true);
      }
      context.clearRect(0, 0, W, H);
      for (const [key, cell] of cells) {
        context.globalAlpha = cell.alpha;
        context.fillStyle = cell.color;
        // Live blips glow; everything else stays flat.
        context.shadowBlur = cell.color === palette.live && cell.alpha > .9 ? 8 : 0;
        context.shadowColor = cell.color;
        context.fillText(cell.char, (key % cols) * cw + cw / 2, Math.floor(key / cols) * ch + ch / 2);
      }
      context.shadowBlur = 0;
      context.globalAlpha = 1;
      if (animate) frame = requestAnimationFrame(draw);
    }
    frame = requestAnimationFrame(draw);
    return () => { disposed = true; cancelAnimationFrame(frame); };
  });
</script>

<canvas class="fleet-radar" bind:this={canvas} width={W} height={H} aria-hidden="true" data-live={live || undefined}></canvas>

<style>
  .fleet-radar { display: block; width: 440px; height: 204px; flex: none; }
</style>
