<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import { onMount } from "svelte";
  import { Terminal } from "@xterm/xterm";
  import { FitAddon } from "@xterm/addon-fit";
  import "@xterm/xterm/css/xterm.css";
  import { workspaceTerminal, type TerminalInfo } from "../../lib/workspace-terminal";
  let { sessionId, domainId, visible }: { sessionId: string; domainId: string; visible: boolean } = $props();
  let host: HTMLDivElement | undefined = $state();
  let terminal: Terminal | undefined;
  let fit: FitAddon | undefined;
  let info = $state<TerminalInfo | null>(null);
  let error = $state("");
  let gap = $state(false);
  let writable = $state(false);
  let ready = $state(false);
  let ending = $state(false);
  let endDialog: HTMLDialogElement | undefined = $state();
  let cursor = 0;
  let inputQueue = Promise.resolve();
  let inputGeneration = 0;
  let fitTimer: ReturnType<typeof setTimeout>;
  function fitVisible() {
    clearTimeout(fitTimer);
    fitTimer = setTimeout(() => {
      if (!visible || !host?.clientWidth || !host.clientHeight || !terminal || !fit) return;
      fit.fit();
      if (info?.state === "running") void workspaceTerminal.resize(sessionId, domainId, Math.max(2, Math.min(300, terminal.rows)), Math.max(10, Math.min(500, terminal.cols))).catch(e => error = String(e));
    }, 80);
  }
  onMount(() => {
    terminal = new Terminal({ cursorBlink: true, disableStdin: true, fontSize: 13, fontFamily: '"IBM Plex Mono", Consolas, monospace', scrollback: 3000, theme: { background: "#101114", foreground: "#e8eaed" }, allowProposedApi: false });
    fit = new FitAddon(); terminal.loadAddon(fit);
    if (host) terminal.open(host);
    ready = true;
    const input = terminal.onData(data => {
      if (!writable || info?.state !== "running") return;
      // Preserve ordering even when IPC responses arrive at different speeds.
      const generation = inputGeneration;
      inputQueue = inputQueue.then(() => { if (generation === inputGeneration && writable) return workspaceTerminal.input(sessionId, domainId, data); }).catch(e => { inputGeneration++; error = String(e); writable = false; });
    });
    const observer = new ResizeObserver(fitVisible); if (host) observer.observe(host);
    fitVisible();
    return () => { writable = false; inputGeneration++; clearTimeout(fitTimer); observer.disconnect(); input.dispose(); terminal?.dispose(); terminal = undefined; };
  });
  $effect(() => { if (visible) fitVisible(); });
  $effect(() => { if (terminal) terminal.options.disableStdin = !writable || info?.state !== "running"; });
  $effect(() => {
    if (!visible || !ready) { writable = false; inputGeneration++; return; }
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    async function poll() {
      try {
        const page = await workspaceTerminal.read(sessionId, domainId, cursor);
        if (disposed) return;
        if (page.session.id !== sessionId || page.session.domain_id !== domainId || page.next < cursor || page.start > page.next || (!page.gap && page.start !== cursor)) throw new Error("Terminal output identity or sequence changed.");
        info = page.session;
        if (page.gap) { gap = true; terminal?.reset(); }
        const bytes = Uint8Array.from(atob(page.data_base64), c => c.charCodeAt(0));
        if (bytes.length !== page.next - page.start) throw new Error("Terminal output byte count changed.");
        cursor = page.next;
        if (bytes.length) await new Promise<void>(resolve => terminal ? terminal.write(bytes, resolve) : resolve());
        if (disposed) return;
        error = page.error ?? "";
        if (info.state !== "running") writable = false;
      } catch (e) { if (!disposed) { error = String(e); writable = false; } }
      if (!disposed) timer = setTimeout(poll, info?.state === "running" ? 150 : 1500);
    }
    void poll(); return () => { disposed = true; clearTimeout(timer); };
  });
  async function end() {
    ending = true; writable = false;
    try { await inputQueue; await workspaceTerminal.end(sessionId, domainId); endDialog?.close(); }
    catch (e) { error = String(e); }
    finally { ending = false; }
  }
</script>

<section class="workspace-terminal" data-workspace-terminal aria-label="Your workspace terminal">
  <header><strong>Your terminal</strong><span>{info?.state ?? "Reconnecting"}{info?.exit_code != null ? ` · exit ${info.exit_code}` : ""}</span></header>
  <p class="scope">Input destination: <code>{domainId}</code><small>Session {sessionId} · You</small></p>
  <p class="authority">Commands edit this workspace directly. Run Apply does not protect these edits.</p>
  {#if error}<p class="error" role="alert">{error}</p>{/if}
  {#if gap}<p role="status">Earlier output exceeded the 1 MiB replay buffer. Terminal display restarted from retained bytes.</p>{/if}
  <div class="terminal-host" bind:this={host}></div>
  <footer><button disabled={info?.state !== "running"} aria-pressed={writable} onclick={() => { writable = !writable; if (writable) terminal?.focus(); }}>{writable ? "Pause input" : "Enable input"}</button><span>{writable ? "Typing goes to this session" : "Input paused"}</span><button disabled={info?.state !== "running"} onclick={() => withPreviewsHidden(() => endDialog?.showModal())}>End session</button></footer>
</section>
<dialog bind:this={endDialog} aria-label="End your terminal session">
  <h2>End this workspace terminal?</h2><p>{domainId}</p><p>This ends your managed shell. Detached background commands may continue. Agent runs are separate.</p>
  {#if error}<p role="alert">{error}</p>{/if}
  <div><button disabled={ending} onclick={() => endDialog?.close()}>Keep session</button><button disabled={ending} onclick={end}>{ending ? "Ending…" : "End this session"}</button></div>
</dialog>
<style>
  .workspace-terminal{display:flex;height:100%;min-height:300px;flex-direction:column;font-size:12px}header,footer{display:flex;align-items:center;flex-wrap:wrap;gap:10px;padding:8px 12px;border-bottom:1px solid var(--pytxo-line-soft)}header span{margin-left:auto;color:var(--pytxo-text-muted)}p{margin:6px 12px;line-height:1.5;overflow-wrap:anywhere}.scope{font-size:11px}.scope small{display:block;color:var(--pytxo-text-muted)}code{font-family:"IBM Plex Mono",monospace}.authority{color:var(--state-unknown);font-size:11px}.error{color:var(--state-refuted)}.terminal-host{flex:1;min-height:120px;overflow:hidden;padding:6px;background:#101114}.terminal-host :global(.xterm){height:100%}footer{border-top:1px solid var(--pytxo-line-soft)}footer span{flex:1;color:var(--pytxo-text-muted)}button{min-height:30px;padding:5px 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-panel);color:var(--pytxo-text-strong);font:inherit;cursor:pointer}button:disabled{opacity:.5;cursor:not-allowed}button:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}dialog{position:fixed;inset:0;margin:auto;max-height:calc(100vh - 32px);overflow:auto;max-width:min(520px,90vw);padding:24px;border:1px solid var(--pytxo-line);border-radius:8px;background:var(--pytxo-surface-raised);color:var(--pytxo-text-strong)}dialog::backdrop{background:#0009}dialog h2{font-size:18px}dialog p{margin:12px 0}dialog>div{display:flex;gap:10px;justify-content:flex-end}
</style>
