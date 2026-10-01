<script lang="ts">
  import SetupStepFrame from "./SetupStepFrame.svelte";
  import { onMount } from "svelte";
  import { betaAdesInOrder, isAdeRunnable, isAdeSessionConfirmed } from "../../lib/ade-status";
  import { createDesktopBackend } from "../../lib/desktop-backend";
  import type { AdeCliStatusDto } from "../../lib/types";
  import { Button } from "$lib/components/ui/button";

  let {
    onRuntimeTools,
    onContinue,
    onSkip,
  }: {
    onRuntimeTools?: () => void;
    onContinue: () => void;
    onSkip: () => void;
  } = $props();

  const backend = createDesktopBackend();
  let agents = $state<AdeCliStatusDto[]>([]);
  let loading = $state(true);
  let openingId = $state<string | null>(null);
  let message = $state("");
  let error = $state("");

  // Onboarding lists the agents this beta can run; other CLIs stay in Setup.
  const usefulAgents = $derived(betaAdesInOrder(agents));
  const installedCount = $derived(usefulAgents.filter((agent) => agent.installed).length);
  const readyCount = $derived(usefulAgents.filter(isReady).length);

  function isReady(agent: AdeCliStatusDto) {
    return isAdeRunnable(agent);
  }

  async function refresh() {
    loading = true;
    agents = [];
    error = "";
    message = "";
    try {
      agents = await backend.listAdeClis();
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      loading = false;
    }
  }

  async function openLogin(agent: AdeCliStatusDto) {
    if (!agent.installed || !agent.login_supported || openingId) return;
    openingId = agent.id;
    message = "";
    error = "";
    try {
      const launched = await backend.startAdeLogin(agent.id);
      message = launched.message;
    } catch (reason) {
      error = reason instanceof Error ? reason.message : String(reason);
    } finally {
      openingId = null;
    }
  }

  onMount(() => {
    void refresh();
  });
</script>

<SetupStepFrame>
  <h2 class="title">Connect your coding agents</h2>
  <p class="lead">
    This beta runs Codex, Claude Code, Cursor Agent, OpenCode and Antigravity with the accounts you already have. Pytxo can open a tool's own sign-in if needed.
  </p>

  {#if loading}
    <p class="status">Finding your coding agents…</p>
  {:else}
    <div class="agents" aria-label="Detected agent CLIs">
      {#each usefulAgents as agent (agent.id)}
        <div class="agent">
          <span>
            <strong>{agent.display_name}</strong>
            <small>{agent.installed ? agent.auth_label : "Not installed"}</small>
          </span>
          {#if agent.installed && agent.auth_state !== "signed_in" && agent.login_supported && agent.login_label}
            <Button size="sm" variant="outline" disabled={openingId !== null} onclick={() => void openLogin(agent)}>
              {openingId === agent.id ? "Opening…" : agent.login_label}
            </Button>
          {:else if !agent.installed}
            <a class="install-guide" href={agent.docs_url} target="_blank" rel="noopener noreferrer">Install guide</a>
          {:else}
            <b class:ready={isReady(agent)}>
              {isAdeSessionConfirmed(agent) ? "Ready" : isReady(agent) ? "Available" : agent.installed ? "Installed" : "Later"}
            </b>
          {/if}
        </div>
      {/each}
    </div>
    {#if agents.length}
      <p class="summary" role="status">{installedCount} installed · {readyCount} available. Confirmed sessions are labeled Ready; vendor-managed sessions stay private.</p>
    {/if}
    {#if readyCount === 0 && !error}<p class="summary">Install an agent or finish sign-in, then check again. You can also finish Desktop setup and connect an agent later.</p>{/if}
  {/if}

  {#if message}<p class="message" role="status">{message}</p>{/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}

  {#snippet actions()}
    {#if onRuntimeTools}<Button variant="ghost" onclick={onRuntimeTools}>Terminal tools</Button>{/if}
    <Button variant="outline" disabled={loading || openingId !== null} onclick={() => void refresh()}>
      {loading ? "Checking…" : "Check again"}
    </Button>
    {#if readyCount > 0}<Button disabled={loading || openingId !== null} onclick={onContinue}>Continue</Button>
    {:else}<Button disabled={loading || openingId !== null} onclick={onSkip}>Set up agents later</Button>{/if}
  {/snippet}
</SetupStepFrame>

<style>
  .install-guide { display: inline-flex; align-items: center; min-height: 40px; padding: 0 10px; border: 1px solid var(--border); border-radius: 6px; color: var(--foreground); text-decoration: none; font-size: 13px; }
  .install-guide:hover { background: var(--accent); }
  .install-guide:focus-visible { outline: 2px solid var(--ring); outline-offset: 2px; }
  .title {
    margin: 0;
    font-size: 1.35rem;
    text-wrap: balance;
  }
  .lead {
    max-width: 490px;
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.9rem;
    line-height: 1.5;
    text-wrap: pretty;
  }
  .agents {
    display: grid;
    width: 100%;
    border: 1px solid var(--border);
    border-radius: var(--panel-radius);
    background: color-mix(in oklab, var(--card) 86%, transparent);
    overflow: hidden;
  }
  .agent {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    min-height: 48px;
    padding: 0.55rem 0.7rem;
    border-bottom: 1px solid var(--border);
    text-align: left;
  }
  .agent:last-child {
    border-bottom: 0;
  }
  .agent > span {
    display: grid;
    gap: 0.16rem;
    min-width: 0;
  }
  .agent strong {
    font-size: 0.82rem;
  }
  .agent small,
  .summary,
  .status,
  .message,
  .error {
    color: var(--muted-foreground);
    font-size: 0.72rem;
  }
  .agent b {
    color: var(--muted-foreground);
    font-size: 0.6875rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .agent b.ready,
  .message {
    color: var(--primary);
  }
  .summary,
  .status,
  .message,
  .error {
    margin: 0;
  }
  .error {
    color: var(--destructive, #f87171);
  }
</style>
