<script lang="ts">
  import { onMount } from "svelte";
  import { localPreview, previewAddress, type PreviewInfo } from "../../lib/local-preview";
  import { hasNativeTerminal } from "../../lib/workspace-terminal";
  import { previewsSuspended, registerPreviewSuspension } from "../../lib/preview-overlay";
  let { visible }: { visible: boolean } = $props();
  let address = $state("");
  let preview = $state<PreviewInfo | null>(null);
  let error = $state("");
  let opening = $state(false);
  let closing = $state(false);
  let paused = $state(false);
  let blockedRequests = $state(0);
  let viewport: HTMLDivElement | undefined = $state();
  const canonical = $derived(previewAddress(address));
  let disposed = false;
  let syncing = false;
  let syncPending = false;
  let revision = 0;
  let sequence = 0;

  async function open() {
    if (!canonical || opening || closing || preview) return;
    opening = true; error = "";
    const generation = ++sequence;
    try {
      const created = await localPreview.open(canonical);
      if (disposed || generation !== sequence) { await localPreview.close(created.id); return; }
      preview = created; paused = false; blockedRequests = 0;
      await sync();
    } catch (cause) { if (!disposed) error = String(cause); }
    finally { opening = false; }
  }
  async function close() {
    if (opening || closing || !preview) return;
    sequence++;
    const current = preview;
    closing = true;
    try {
      await localPreview.close(current.id);
      if (!disposed && preview?.id === current.id) { preview = null; error = ""; }
    } catch (cause) { if (!disposed && preview?.id === current.id) error = String(cause); }
    finally { closing = false; }
  }
  async function retry() {
    if (!preview || opening || closing) return;
    const previous = preview;
    const generation = ++sequence;
    opening = true;
    try {
      // Never open another renderer unless the old one acknowledged closure.
      await localPreview.close(previous.id);
      if (disposed || generation !== sequence) return;
      preview = null; error = "";
      const created = await localPreview.open(previous.url);
      if (disposed || generation !== sequence) { await localPreview.close(created.id); return; }
      preview = created; paused = false; blockedRequests = 0;
      await sync();
    } catch (cause) { if (!disposed) error = String(cause); }
    finally { opening = false; }
  }
  async function sync() {
    if (!preview || disposed) return;
    if (syncing) { syncPending = true; return; }
    syncing = true;
    const id = preview.id;
    try {
      const rect = viewport?.getBoundingClientRect();
      const blocked = previewsSuspended() || !!document.querySelector('dialog[open], [role="dialog"], [role="alertdialog"], .workspace-tools details[open], .panel-tools details[open]');
      const bounds = visible && !paused && !blocked && document.visibilityState === "visible" && rect && rect.width >= 80 && rect.height >= 60 && viewport?.checkVisibility()
        ? { x: rect.x, y: rect.y, width: rect.width, height: rect.height, viewport_width: innerWidth, viewport_height: innerHeight } : null;
      const result = await localPreview.sync(id, bounds, ++revision);
      if (preview?.id === id) {
        blockedRequests = result.blocked_requests;
        if (result.failure) { if (!error) error = result.failure; paused = true; }
      }
    } catch (cause) {
      // A closed renderer may reject after Retry already created its replacement.
      if (!disposed && preview?.id === id) { error = String(cause); paused = true; }
    }
    finally { syncing = false; if (syncPending) { syncPending = false; queueMicrotask(() => void sync()); } }
  }
  // DOM overlays cannot paint above a native child. Hide on parent interaction
  // and resume only after overlays close; a native lease also covers renderer stalls.
  onMount(() => {
    disposed = false;
    const observer = new MutationObserver(() => void sync());
    observer.observe(document.body, { subtree: true, attributes: true, attributeFilter: ["open", "hidden", "class", "style"], childList: true });
    const resize = new ResizeObserver(() => void sync());
    if (viewport) resize.observe(viewport);
    const hide = async () => {
      const current = preview;
      if (current) {
        try { await localPreview.sync(current.id, null, ++revision); }
        catch {
          if (disposed || preview?.id !== current.id) return;
          paused = true;
          try {
            await localPreview.close(current.id);
            if (disposed || preview?.id !== current.id) return;
            preview = null;
            error = "The preview stopped responding and was closed. Your local server is still running.";
          } catch {
            if (disposed || preview?.id !== current.id) return;
            error = "Pytxo could not hide or close the preview. The dialog has not opened. Wait for the native app to respond, then retry.";
            throw new Error(error);
          }
        }
      }
    };
    const unregister = registerPreviewSuspension(hide);
    // hide already reports errors for the current renderer; do not reapply a stale rejection.
    const hideBeforeInteraction = () => { void hide().catch(() => {}); };
    document.addEventListener("pointerdown", hideBeforeInteraction, true);
    document.addEventListener("keydown", hideBeforeInteraction, true);
    window.addEventListener("hashchange", hideBeforeInteraction);
    const timer = setInterval(() => void sync(), 200);
    return () => {
      disposed = true; sequence++;
      unregister();
      observer.disconnect(); resize.disconnect(); clearInterval(timer);
      document.removeEventListener("pointerdown", hideBeforeInteraction, true);
      document.removeEventListener("keydown", hideBeforeInteraction, true);
      window.removeEventListener("hashchange", hideBeforeInteraction);
      if (preview) void localPreview.close(preview.id).catch(() => {});
    };
  });
  $effect(() => { void visible; void paused; void sync(); });
</script>

<section class="local-preview" aria-label="Local preview">
  <div class="preview-controls">
    {#if preview}
      <div class="running-tools"><div class="address" title={preview.url}>{preview.url}</div>{#if error && paused}<button disabled={opening || closing} onclick={retry}>{opening ? "Retrying…" : "Retry preview"}</button>{:else}<button disabled={opening || closing} onclick={() => paused = !paused}>{paused ? "Resume preview" : "Pause preview"}</button>{/if}<button disabled={opening || closing} onclick={close}>{closing ? "Closing…" : "Close preview"}</button></div>
      <div class="boundary-row"><span>Local page · unverified</span><details><summary>Preview boundary</summary><p>Separate storage · no Pytxo commands. Other servers and external resources are blocked. F6 returns keyboard focus to Pytxo.</p></details></div>
      {#if blockedRequests}<p class="blocked" role="status">{blockedRequests} requests blocked · external assets or services may be missing.</p>{/if}
    {:else}
      <form onsubmit={event => { event.preventDefault(); void open(); }}>
        <label>Local server URL<input type="url" placeholder="http://localhost:5173" bind:value={address} disabled={opening} spellcheck="false" autocomplete="off" /></label>
        {#if canonical}<p class="address">Open {canonical}</p>{:else if address}<p>Enter a complete localhost, 127.0.0.1 or [::1] address.</p>{/if}
        <button disabled={!canonical || opening || !hasNativeTerminal()}>{opening ? "Opening…" : "Open preview"}</button>
      </form>
      <p>Start your server yourself, then open its address here. Layout restore never opens a page or starts a server.</p>
      {#if !hasNativeTerminal()}<p>Local previews require native Windows Desktop.</p>{/if}
    {/if}
    {#if error}<p class="error" role="alert">{error}</p>{/if}
  </div>
  <div class="preview-viewport" bind:this={viewport} aria-label="Local page viewport">
    <span>{preview ? paused ? "Preview paused" : "Local page appears here when the dock is visible and menus are closed." : "Your local app, alongside the mission."}</span>
  </div>
</section>

<style>
  .local-preview{display:flex;flex-direction:column;flex:1;min-height:0;color:var(--pytxo-text-strong)}
  .preview-controls{padding:10px 12px;flex-shrink:0;border-bottom:1px solid var(--pytxo-line-soft)}
  form,label{display:flex;flex-direction:column;gap:8px}label{font-size:12px}
  input{width:100%;min-width:0;min-height:32px;padding:6px 8px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-panel);color:inherit;font:inherit}
  button{min-height:28px;padding:4px 8px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-raised);color:inherit;font:inherit;font-size:12px;cursor:pointer}button:disabled{opacity:.5;cursor:default}
  p{margin:8px 0 0;color:var(--pytxo-text-muted);font-size:11px;line-height:1.5}.address{font-size:12px;overflow-wrap:anywhere}.error{color:var(--state-refuted)}
  .running-tools{display:flex;align-items:center;flex-wrap:wrap;gap:6px}.running-tools .address{flex:1;min-width:110px;overflow:hidden;white-space:nowrap;text-overflow:ellipsis}.running-tools button{flex-shrink:0}
  .boundary-row{display:flex;align-items:baseline;flex-wrap:wrap;gap:6px;margin-top:5px;font-size:11px;color:var(--pytxo-text-muted)}.boundary-row details{margin-left:auto}.boundary-row details[open]{flex-basis:100%}.boundary-row summary{cursor:pointer}.blocked{margin-top:4px}.boundary-row summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
  .preview-viewport{flex:1;min-height:80px;display:grid;place-items:center;margin:2px;background:var(--pytxo-surface-inset);overflow:hidden}.preview-viewport span{max-width:28em;padding:20px;text-align:center;font-size:12px;color:var(--pytxo-text-muted)}
  button:focus-visible,input:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
</style>
