<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { IconCopy, IconWindowMaximize, IconWindowMinimize, IconX } from "@tabler/icons-svelte";

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

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar__brand" data-tauri-drag-region>
    <img src="/logo-mark.png" alt="" width="16" height="16" class="titlebar__logo" />
    <span class="titlebar__name">Pytxo Desktop</span>
  </div>
  <div class="titlebar__controls">
    <button type="button" class="titlebar__btn" onclick={minimize} aria-label="Minimize">
      <IconWindowMinimize size={15} stroke={1.75} />
    </button>
    <button
      type="button"
      class="titlebar__btn"
      onclick={toggleMaximize}
      aria-label={isMaximized ? "Restore" : "Maximize"}
    >
      {#if isMaximized}
        <IconCopy size={14} stroke={1.75} />
      {:else}
        <IconWindowMaximize size={14} stroke={1.75} />
      {/if}
    </button>
    <button type="button" class="titlebar__btn titlebar__btn--close" onclick={close} aria-label="Close">
      <IconX size={16} stroke={1.75} />
    </button>
  </div>
</header>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 36px;
    padding: 0 0 0 0.75rem;
    background: var(--sidebar);
    border-bottom: 1px solid var(--sidebar-border);
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
