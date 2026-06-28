<script lang="ts">
  import { onMount } from "svelte";
  import { ipc } from "../../lib/ipc";

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

  async function refresh() {
    const present = await ipc.checkPytxoCli();
    if (present) {
      phase = "done";
      message = "Pytxo CLI is on your PATH.";
    } else if (phase !== "installing") {
      phase = "ready";
      message =
        "Install the CLI for terminal parity, MCP tools, and pytxo doctor. Dispatch works without it.";
    }
  }

  async function install() {
    phase = "installing";
    message = "Downloading from GitHub Releases…";
    try {
      const status = await ipc.installPytxoCli();
      message = status.message;
      if (status.cli_present) {
        phase = "done";
      } else {
        phase = "error";
      }
    } catch (e) {
      phase = "error";
      message = String(e);
    }
  }

  onMount(refresh);
</script>

<div class="step">
  <h2 class="title">Pytxo CLI</h2>
  <p class="lead">{message}</p>

  {#if phase === "checking" || phase === "installing"}
    <div class="spinner" aria-hidden="true"></div>
  {/if}

  <div class="actions">
    {#if phase === "ready" || phase === "error"}
      <button type="button" class="primary" onclick={install}>Install Pytxo CLI</button>
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
      <button type="button" class="primary" onclick={onContinue}>Continue</button>
    {/if}
    {#if phase !== "installing"}
      <button type="button" class="ghost" onclick={onSkip}>Skip for now</button>
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
  .spinner {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    border: 2px solid color-mix(in oklab, var(--brand-teal) 25%, transparent);
    border-top-color: var(--brand-teal);
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
  button {
    min-height: 44px;
    border-radius: 10px;
  }
  .ghost {
    background: transparent;
    border: none;
    color: var(--muted-foreground);
  }
  .link {
    font-size: 0.85rem;
    color: var(--brand-teal);
  }
</style>
