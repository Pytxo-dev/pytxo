<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let {
    title = "Pytxo Desktop",
  }: {
    title?: string;
  } = $props();

  let isMaximized = $state(false);

  function currentWindow() {
    if (typeof window === "undefined" || !("__TAURI_INTERNALS__" in window)) return null;
    return getCurrentWindow();
  }

  async function refreshMaximized() {
    const win = currentWindow();
    if (!win) return;
    try {
      isMaximized = await win.isMaximized();
    } catch {
      /* older webview without isMaximized support */
    }
  }

  onMount(() => {
    void refreshMaximized();
    const win = currentWindow();
    if (!win) return;
    let unlisten: (() => void) | undefined;
    win
      .onResized(() => {
        void refreshMaximized();
      })
      .then((fn) => {
        unlisten = fn;
      });
    return () => unlisten?.();
  });

  async function minimize() {
    await currentWindow()?.minimize();
  }

  async function toggleMaximize() {
    await currentWindow()?.toggleMaximize();
    await refreshMaximized();
  }

  async function close() {
    await currentWindow()?.close();
  }
</script>

<div class="titlebar-wrap">
<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar__brand" data-tauri-drag-region>
    <img src="/logo-mark.png" alt="" width="16" height="16" class="titlebar__logo" />
    <span class="titlebar__name">{title}</span>
  </div>
  <div class="titlebar__controls">
    <!-- Windows caption glyphs: 10px, 1px strokes, drawn rather than borrowed from an icon set. -->
    <button type="button" class="titlebar__btn" onclick={minimize} aria-label="Minimize">
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M0 5.5h10" /></svg>
    </button>
    <button
      type="button"
      class="titlebar__btn"
      onclick={toggleMaximize}
      aria-label={isMaximized ? "Restore" : "Maximize"}
    >
      {#if isMaximized}
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2.5 2.5V.5h7v7h-2M.5 2.5h7v7h-7z" /></svg>
      {:else}
        <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M.5.5h9v9h-9z" /></svg>
      {/if}
    </button>
    <button type="button" class="titlebar__btn titlebar__btn--close" onclick={close} aria-label="Close">
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M.5.5l9 9M9.5.5l-9 9" /></svg>
    </button>
  </div>
</header>
<span class="chroma-ribbon" aria-hidden="true"></span>
</div>

<style>
  .titlebar-wrap {
    flex-shrink: 0;
  }
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 0 0 0.75rem;
    background: var(--sidebar);
    border-bottom: 0;
    flex-shrink: 0;
    user-select: none;
  }
  .titlebar__brand {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .titlebar__logo {
    flex-shrink: 0;
  }
  .titlebar__name {
    font-size: 0.75rem;
    font-weight: 500;
    color: var(--muted-foreground);
    letter-spacing: 0.01em;
  }
  .titlebar__controls {
    display: flex;
    height: 100%;
    /* Window controls must never be part of the drag region. WebView2 is
       Chromium-based, so the recognized property is vendor-prefixed. */
    -webkit-app-region: no-drag;
  }
  .titlebar__btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    border: none;
    border-radius: 0;
    background: transparent;
    color: var(--muted-foreground);
    line-height: 1;
    min-height: unset;
    box-shadow: none;
    padding: 0;
    transition: background-color 150ms ease, color 150ms ease;
  }
  .titlebar__btn svg {
    width: 10px;
    height: 10px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1;
  }
  .titlebar__btn:hover {
    background: color-mix(in oklab, var(--foreground) 8%, transparent);
    color: var(--foreground);
  }
  .titlebar__btn:focus-visible {
    outline: 2px solid var(--ring);
    outline-offset: -2px;
  }
  .titlebar__btn--close:hover {
    background: var(--destructive);
    color: #fff;
  }
</style>
