<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { ipc } from "../../lib/ipc";
  import { Button } from "$lib/components/ui/button";

  let {
    onContinue,
    onSkip,
  }: {
    onContinue: () => void;
    onSkip: () => void;
  } = $props();

  type Phase = "checking" | "ready" | "installing" | "done" | "error";

  let phase = $state<Phase>("checking");
  let message = $state("");
  let pathPending = $state(false);
  let pollTimer: ReturnType<typeof setTimeout> | null = null;
  let pollInflight = false;
  let disposed = false;

  async function refresh() {
    const present = await ipc.checkPytxoCli();
    if (present) {
      phase = "done";
      message = "Pytxo CLI is ready.";
      pathPending = false;
    } else if (phase !== "installing") {
      phase = "ready";
      message =
        "Desktop can run work without a separate CLI install. Install the CLI for terminal workflows, MCP tools, and pytxo doctor.";
    }
  }

  function stopPoll() {
    if (pollTimer) {
      clearTimeout(pollTimer);
      pollTimer = null;
    }
  }

  function schedulePoll() {
    stopPoll();
    if (disposed || phase !== "installing") return;
    pollTimer = setTimeout(() => {
      void pollOnce().finally(() => {
        if (!disposed && phase === "installing") schedulePoll();
      });
    }, 500);
  }

  async function pollOnce() {
    if (pollInflight) return;
    pollInflight = true;
    try {
      const st = await ipc.installPytxoCliStatus();
      if (st.message) message = st.message;
    } catch {
      /* ignore poll errors */
    } finally {
      pollInflight = false;
    }
  }

  async function install() {
    phase = "installing";
    message = "Downloading from GitHub Releases…";
    pathPending = false;
    schedulePoll();
    try {
      const status = await ipc.installPytxoCli();
      message = status.message;
      pathPending = status.path_pending;
      if (status.cli_present) {
        phase = "done";
      } else {
        phase = "error";
      }
    } catch (e) {
      phase = "error";
      message = String(e);
    } finally {
      stopPoll();
    }
  }

  onMount(() => {
    disposed = false;
    void refresh();
    return () => {
      disposed = true;
      stopPoll();
    };
  });
  onDestroy(() => {
    disposed = true;
    stopPoll();
  });
</script>

<div class="step">
  <h2 class="title">Terminal tools are optional</h2>
  <p class="hint">Desktop includes the local Pytxo core.</p>
  <p class="lead">{message}</p>
  {#if pathPending && phase === "done"}
    <p class="hint">
      CLI installed. Restart this shell (or open a new terminal) so <code>pytxo</code> appears on PATH.
    </p>
  {/if}

  {#if phase === "checking" || phase === "installing"}
    <div class="spinner" aria-hidden="true"></div>
  {/if}

  <div class="actions">
    {#if phase !== "installing" && phase !== "done"}
      <Button onclick={onSkip}>Continue with Desktop</Button>
    {/if}
    {#if phase === "ready" || phase === "error"}
      <Button variant="outline" onclick={install}>Install Pytxo CLI</Button>
      <a
        class="link"
        href="https://pytxo.com/download"
        target="_blank"
        rel="noopener noreferrer"
      >
        Manual download
      </a>
    {/if}
    {#if phase === "done"}
      <Button onclick={onContinue}>Continue</Button>
    {/if}
  </div>
</div>

<style>
  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1rem;
    max-width: 440px;
    margin: 0 auto;
  }
  .title {
    margin: 0;
    font-size: 1.35rem;
    text-wrap: balance;
  }
  .lead {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.9rem;
    text-wrap: pretty;
    line-height: 1.5;
  }
  .hint {
    margin: 0;
    font-size: 0.8rem;
    color: var(--muted-foreground);
  }
  .spinner {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid color-mix(in oklab, var(--primary) 25%, transparent);
    border-top-color: var(--primary);
    animation: spin 0.8s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .actions {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    width: 100%;
    max-width: 280px;
  }
  .actions :global(button) {
    width: 100%;
  }
  .link {
    font-size: 0.85rem;
    color: var(--primary);
  }
</style>
