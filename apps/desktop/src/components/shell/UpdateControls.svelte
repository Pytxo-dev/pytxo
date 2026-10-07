<script lang="ts">
  import { desktopUpdater } from "$lib/desktop-updater";
  import {
    isUpdateBusy,
    isUpdateCheckDisabled,
    shouldShowUpdateCheck,
    updateActionLabel,
    updatePhaseMessage,
  } from "$lib/update-presentation";
  let { compact = false } = $props<{ compact?: boolean }>();
  const busy = $derived(isUpdateBusy($desktopUpdater));
  const checkDisabled = $derived(isUpdateCheckDisabled($desktopUpdater));
  const showCheck = $derived(shouldShowUpdateCheck($desktopUpdater, compact));
  const phaseMessage = $derived(updatePhaseMessage($desktopUpdater));
</script>

<div class="update-controls" aria-busy={busy}>
  <div class="update-copy" role="status" aria-live="polite">
    {#if $desktopUpdater.currentVersion}<span class="update-version">Desktop v{$desktopUpdater.currentVersion}</span>{/if}
    <span class="update-message">{phaseMessage}</span>
    {#if $desktopUpdater.notice}<span>{$desktopUpdater.notice}</span>{/if}
    {#if $desktopUpdater.error}<span class="update-error" role="alert">{$desktopUpdater.error}</span>{/if}
  </div>
  <div class="update-actions">
    {#if showCheck}
      <button class:update-primary={!checkDisabled} disabled={checkDisabled} onclick={() => void desktopUpdater.check()}>Check for updates</button>
    {/if}
    {#if $desktopUpdater.version}
      <button class="update-primary" disabled={busy} onclick={() => void desktopUpdater.install()}>{updateActionLabel($desktopUpdater)}</button>
    {/if}
    {#if $desktopUpdater.error}<a class="update-manual" href="https://pytxo.com/download" target="_blank" rel="noopener noreferrer">Manual download</a>{/if}
  </div>
</div>

<style>
  .update-controls { display: flex; flex-wrap: wrap; align-items: center; gap: 12px; width: 100%; }
  .update-copy { display: grid; gap: 4px; flex: 1 1 240px; font-size: 12px; color: var(--pytxo-text-soft); overflow-wrap: anywhere; }
  .update-version { color: var(--pytxo-text-muted); font-family: "IBM Plex Mono", monospace; font-variant-numeric: tabular-nums; }
  .update-message { color: var(--pytxo-text-body); }
  .update-actions { display: flex; flex-wrap: wrap; gap: 8px; align-items: center; }
  button, a { min-height: 36px; display: inline-flex; align-items: center; justify-content: center; font: inherit; font-size: 12px; font-weight: 500; color: var(--pytxo-text-strong); padding: 8px 12px; border: 1px solid var(--pytxo-line); border-radius: 6px; background: transparent; text-decoration: none; cursor: pointer; transition: background var(--pytxo-motion-fast) var(--pytxo-motion-ease), border-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), color var(--pytxo-motion-fast) var(--pytxo-motion-ease); }
  button:hover:not(:disabled), a:hover { border-color: color-mix(in oklab, var(--pytxo-text-muted) 55%, var(--pytxo-line)); background: var(--pytxo-surface-hover); }
  button:focus-visible, a:focus-visible { outline: 2px solid var(--pytxo-text-strong); outline-offset: 2px; }
  .update-primary { border-color: var(--primary); background: var(--primary); color: var(--primary-foreground); }
  .update-primary:hover:not(:disabled) { border-color: color-mix(in oklab, var(--primary) 84%, var(--pytxo-text-muted)); background: color-mix(in oklab, var(--primary) 90%, var(--pytxo-surface-panel)); }
  .update-manual { padding-inline: 8px; border-color: transparent; color: var(--pytxo-text-soft); }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .update-error { color: var(--destructive); }
  @media (prefers-reduced-motion: reduce) { button, a { transition: none; } }
</style>
