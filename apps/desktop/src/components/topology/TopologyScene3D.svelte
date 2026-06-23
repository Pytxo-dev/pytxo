<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import type { StructuralGraphDto } from "../../lib/types";

  let {
    structural = null as StructuralGraphDto | null,
    selectedNodeId = $bindable(null as string | null),
    lightTheme = false,
  }: {
    structural?: StructuralGraphDto | null;
    selectedNodeId?: string | null;
    lightTheme?: boolean;
  } = $props();

  let containerEl: HTMLDivElement | undefined = $state();

  let renderer: THREE.WebGLRenderer | null = null;
  let scene: THREE.Scene | null = null;
  let camera: THREE.PerspectiveCamera | null = null;
  let controls: OrbitControls | null = null;
  let frameId = 0;
  let nodeMeshes = new Map<string, THREE.Mesh>();
  let resizeObserver: ResizeObserver | null = null;

  function token(name: string, fallback: string): string {
    if (typeof getComputedStyle === "undefined") return fallback;
    const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
  }

  function hexColor(cssVar: string, fallback: number): number {
    const raw = token(cssVar, "");
    if (!raw) return fallback;
    try {
      return new THREE.Color(raw).getHex();
    } catch {
      return fallback;
    }
  }

  function layout3d(
    nodes: StructuralGraphDto["nodes"],
    radius: number,
  ): Map<string, THREE.Vector3> {
    const positions = new Map<string, THREE.Vector3>();
    const n = nodes.length;
    nodes.forEach((node, i) => {
      const phi = Math.acos(-1 + (2 * i) / Math.max(n, 1));
      const theta = Math.sqrt(n * Math.PI) * phi;
      positions.set(
        node.id,
        new THREE.Vector3(
          radius * Math.cos(theta) * Math.sin(phi),
          radius * Math.sin(theta) * Math.sin(phi),
          radius * Math.cos(phi),
        ),
      );
    });
    return positions;
  }

  function clearSceneNodes() {
    if (!scene) return;
    for (const mesh of nodeMeshes.values()) {
      scene.remove(mesh);
      mesh.geometry.dispose();
      (mesh.material as THREE.Material).dispose();
    }
    nodeMeshes.clear();
    const toRemove = scene.children.filter((c: THREE.Object3D) => c.userData.kind === "edge");
    for (const obj of toRemove) {
      scene.remove(obj);
      if (obj instanceof THREE.Line) {
        obj.geometry.dispose();
        (obj.material as THREE.Material).dispose();
      }
    }
  }

  function rebuildGraph() {
    if (!scene) return;
    clearSceneNodes();

    const graph = structural;
    if (!graph || graph.nodes.length === 0) return;

    const positions = layout3d(graph.nodes, 4);
    const teal = hexColor("--brand-teal", 0x2dd4bf);
    const violet = hexColor("--brand-violet", 0xa78bfa);
    const gold = hexColor("--brand-gold", 0xfbbf24);

    for (const node of graph.nodes) {
      const pos = positions.get(node.id);
      if (!pos) continue;
      const size = node.edited ? 0.35 : 0.22;
      const color = node.edited ? gold : teal;
      const geom = new THREE.SphereGeometry(size, 16, 16);
      const mat = new THREE.MeshStandardMaterial({
        color,
        emissive: node.edited ? gold : violet,
        emissiveIntensity: node.edited ? 0.35 : 0.12,
      });
      const mesh = new THREE.Mesh(geom, mat);
      mesh.position.copy(pos);
      mesh.userData = { nodeId: node.id, label: node.label };
      scene.add(mesh);
      nodeMeshes.set(node.id, mesh);
    }

    const edgeMat = new THREE.LineBasicMaterial({
      color: hexColor("--brand-violet", 0xa78bfa),
      transparent: true,
      opacity: 0.55,
    });
    for (const edge of graph.edges) {
      const a = positions.get(edge.from);
      const b = positions.get(edge.to);
      if (!a || !b) continue;
      const geom = new THREE.BufferGeometry().setFromPoints([a, b]);
      const line = new THREE.Line(geom, edgeMat);
      line.userData = { kind: "edge" };
      scene.add(line);
    }

    highlightSelection();
  }

  function highlightSelection() {
    for (const [id, mesh] of nodeMeshes) {
      const mat = mesh.material as THREE.MeshStandardMaterial;
      mat.emissiveIntensity = id === selectedNodeId ? 0.65 : 0.12;
      mesh.scale.setScalar(id === selectedNodeId ? 1.25 : 1);
    }
  }

  function applySceneTheme() {
    if (!scene) return;
    const bg = lightTheme ? token("--void", "#f4f6f8") : token("--void", "#020205");
    scene.background = new THREE.Color(bg);
  }

  function resize() {
    if (!containerEl || !renderer || !camera) return;
    const w = containerEl.clientWidth;
    const h = containerEl.clientHeight;
    if (w < 1 || h < 1) return;
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h, false);
  }

  function onPointerDown(ev: PointerEvent) {
    if (!camera || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    const x = ((ev.clientX - rect.left) / rect.width) * 2 - 1;
    const y = -((ev.clientY - rect.top) / rect.height) * 2 + 1;
    const raycaster = new THREE.Raycaster();
    raycaster.setFromCamera(new THREE.Vector2(x, y), camera);
    const hits = raycaster.intersectObjects([...nodeMeshes.values()]);
    if (hits.length > 0) {
      const id = hits[0].object.userData.nodeId as string;
      selectedNodeId = selectedNodeId === id ? null : id;
      highlightSelection();
    }
  }

  function animate() {
    frameId = requestAnimationFrame(animate);
    controls?.update();
    renderer?.render(scene!, camera!);
  }

  onMount(() => {
    if (!containerEl) return;

    scene = new THREE.Scene();
    applySceneTheme();

    camera = new THREE.PerspectiveCamera(50, 1, 0.1, 100);
    camera.position.set(0, 0, 10);

    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    containerEl.appendChild(renderer.domElement);

    const ambient = new THREE.AmbientLight(0xffffff, 0.65);
    const dir = new THREE.DirectionalLight(0xffffff, 0.85);
    dir.position.set(4, 6, 8);
    scene.add(ambient, dir);

    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.minDistance = 3;
    controls.maxDistance = 24;

    resizeObserver = new ResizeObserver(resize);
    resizeObserver.observe(containerEl);
    containerEl.addEventListener("pointerdown", onPointerDown);
    resize();
    rebuildGraph();
    animate();
  });

  onDestroy(() => {
    cancelAnimationFrame(frameId);
    resizeObserver?.disconnect();
    containerEl?.removeEventListener("pointerdown", onPointerDown);
    clearSceneNodes();
    controls?.dispose();
    renderer?.dispose();
  });

  $effect(() => {
    structural;
    rebuildGraph();
  });

  $effect(() => {
    selectedNodeId;
    highlightSelection();
  });

  $effect(() => {
    lightTheme;
    applySceneTheme();
  });
</script>

<section class="topology-3d glass-panel chroma-edge-top">
  <header class="topology-3d__header">
    <h2 class="panel-title">AST topology (3D)</h2>
    {#if selectedNodeId}
      <span class="selection">{selectedNodeId}</span>
    {/if}
  </header>
  <div class="topology-3d__canvas chroma-border" bind:this={containerEl}></div>
  {#if !structural || structural.nodes.length === 0}
    <p class="empty">Select a run with structural telemetry to populate the graph.</p>
  {/if}
</section>

<style>
  .topology-3d {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
    padding: 0.75rem;
    margin: 0 0.5rem 0.5rem 0;
  }
  .topology-3d__header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    margin-bottom: 0.5rem;
  }
  .selection {
    font-size: 0.75rem;
    color: var(--brand-violet);
    font-family: ui-monospace, monospace;
  }
  .topology-3d__canvas {
    flex: 1;
    min-height: 240px;
    border-radius: 8px;
    overflow: hidden;
    background: var(--void-elevated);
  }
  .topology-3d__canvas :global(canvas) {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }
  .empty {
    font-size: 0.8rem;
    opacity: 0.7;
    margin: 0.35rem 0 0;
  }
</style>
