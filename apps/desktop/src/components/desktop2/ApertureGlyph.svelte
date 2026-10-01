<script lang="ts">
  import { onMount } from "svelte";
  import { uiPrefs } from "../../lib/ui-prefs.svelte";
  let { active = false, tone = "unknown" }: { active?: boolean; tone?: "active" | "settled" | "unknown" | "refuted" } = $props();
  let canvas: HTMLCanvasElement;
  let visible = $state(false);
  let hidden = $state(false);
  let reduced = $state(false);
  let ready = $state(false);
  let themeRevision = $state(0);
  // Ambient motion is brand presence, not execution evidence. data-active and
  // the surrounding run label retain the actual worker state.
  const moving = $derived(visible && !hidden && !reduced && !uiPrefs.reducedMotion && ready);
  const speed = $derived(active ? .55 : .12);
  const shell = Array.from({ length: 340 }, (_, n) => {
    const y = 1 - 2 * (n + .5) / 340, radius = Math.sqrt(1 - y * y);
    const angle = n * Math.PI * (3 - Math.sqrt(5));
    return { x: Math.cos(angle) * radius, y, z: Math.sin(angle) * radius, char: ".:+*=x"[n % 6] };
  });
  const points = Array.from({ length: 17 * 17 }, (_, n) => {
    const i = n % 17 - 8, j = Math.floor(n / 17) - 8;
    const x = i / 8, y = j / 8, radius = x * x + y * y;
    return { x, y, px: 96 + i * 10.2, py: 96 + j * 10.2, z: Math.sqrt(Math.max(0, 1 - radius)), inside: radius <= 1,
      char: ".:+*=x"[Math.abs(i * 7 + j * 11) % 6], color: x > .5 && y > .3 ? 3 : y < -.3 ? 0 : y < .4 ? 1 : 2 };
  }).filter(p => p.inside);
  const colors = ["--pytxo-glyph-cyan", "--pytxo-glyph-periwinkle", "--pytxo-glyph-violet", "--pytxo-glyph-warm"];
  const characters = ".:+*=x";
  function createAtlas(palette: string[]) {
    const cell = 18;
    const atlas = document.createElement("canvas");
    atlas.width = characters.length * cell;
    atlas.height = palette.length * cell;
    const context = atlas.getContext("2d")!;
    context.font = '500 14px "IBM Plex Mono", monospace';
    context.textAlign = "center";
    context.textBaseline = "middle";
    palette.forEach((color, row) => {
      context.fillStyle = color;
      [...characters].forEach((character, column) => context.fillText(character, column * cell + cell / 2, row * cell + cell / 2));
    });
    return atlas;
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
    document.fonts.ready.then(() => { if (alive) ready = !!canvas.getContext("2d"); });
    return () => { alive = false; observer.disconnect(); themeObserver.disconnect(); document.removeEventListener("visibilitychange", sync); media.removeEventListener("change", sync); };
  });
  $effect(() => {
    void themeRevision;
    void tone;
    if (!moving) return;
    const context = canvas.getContext("2d");
    if (!context) return;
    const style = getComputedStyle(canvas);
    const palette = colors.map(name => style.getPropertyValue(name).trim());
    const atlas = createAtlas(palette);
    const cadence = speed;
    const frameInterval = active ? 1000 / 30 : 1000 / 12;
    let frame = 0, previous = 0, phase = 0, disposed = false;
    function draw(time: number) {
      if (disposed || !context) return;
      if (previous && time - previous < frameInterval) { frame = requestAnimationFrame(draw); return; }
      phase += previous ? Math.min(time - previous, 100) / 1000 * cadence : 0;
      previous = time;
      context.clearRect(0, 0, 192, 192);
      for (const p of shell) {
        const x = p.x * Math.cos(phase) + p.z * Math.sin(phase);
        const z = p.z * Math.cos(phase) - p.x * Math.sin(phase);
        if (z < -.15 || Math.abs(p.y - .46 * x) < .18) continue;
        context.globalAlpha = .3 + .7 * Math.max(0, z);
        const color = p.y < -.35 ? 0 : p.y < .25 ? 1 : x > .45 ? 3 : 2;
        const character = characters.indexOf(p.char);
        context.drawImage(atlas, character * 18, color * 18, 18, 18, 87 + x * 80, 87 + p.y * 80, 18, 18);
      }
      frame = requestAnimationFrame(draw);
    }
    frame = requestAnimationFrame(draw);
    return () => { disposed = true; cancelAnimationFrame(frame); context.clearRect(0, 0, 192, 192); };
  });
</script>

<div class="aperture-glyph" data-active={active && moving} data-animated={moving} data-motion={active ? "execution" : "ambient"} data-tone={tone} aria-hidden="true">
  <svg viewBox="0 0 192 192" class:concealed={moving}>
    {#each points.filter(p => Math.abs(p.y - .46 * p.x) >= .17) as p}
      <text x={p.px} y={p.py} fill={`var(${colors[p.color]})`} opacity={.65 + .35 * p.z}>{p.char}</text>
    {/each}
  </svg>
  <canvas bind:this={canvas} width="192" height="192" class:concealed={!moving}></canvas>
</div>

<style>
  .aperture-glyph { width: 64px; height: 64px; position: relative; flex: 0 0 64px; }
  svg,canvas { position: absolute; inset: 0; width: 100%; height: 100%; }
  text { font: 500 14px "IBM Plex Mono", monospace; text-anchor: middle; dominant-baseline: central; }
  .concealed { visibility: hidden; }
  [data-tone="unknown"] { filter: saturate(.2); }
  [data-tone="refuted"] { --pytxo-glyph-warm: var(--state-refuted); }
</style>
