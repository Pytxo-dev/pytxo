<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  const win = getCurrentWindow();

  async function minimize() {
    await win.minimize();
  }

  async function toggleMaximize() {
    await win.toggleMaximize();
  }

  async function close() {
    await win.close();
  }
</script>

<header class="titlebar" data-tauri-drag-region>
  <div class="titlebar__brand" data-tauri-drag-region>
    <img src="/logo.png" alt="" width="18" height="18" class="titlebar__logo" />
    <span class="titlebar__name">Pytxo Desktop</span>
  </div>
  <div class="titlebar__controls">
    <button type="button" class="titlebar__btn" onclick={minimize} aria-label="Minimize">—</button>
    <button type="button" class="titlebar__btn" onclick={toggleMaximize} aria-label="Maximize">□</button>
    <button type="button" class="titlebar__btn titlebar__btn--close" onclick={close} aria-label="Close">×</button>
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
    border-radius: 4px;
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
  }
  .titlebar__btn {
    width: 46px;
    height: 100%;
    border: none;
    border-radius: 0;
    background: transparent;
    color: var(--muted-foreground);
    font-size: 0.85rem;
    line-height: 1;
    min-height: unset;
    box-shadow: none;
    padding: 0;
  }
  .titlebar__btn:hover {
    background: color-mix(in oklab, var(--foreground) 8%, transparent);
    color: var(--foreground);
  }
  .titlebar__btn--close:hover {
    background: #c42b1c;
    color: #fff;
  }
</style>
