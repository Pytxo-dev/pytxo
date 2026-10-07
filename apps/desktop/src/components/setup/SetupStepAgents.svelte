<script lang="ts">
  import SetupStepFrame from "./SetupStepFrame.svelte";
  import AdeIdentity from "../desktop2/AdeIdentity.svelte";
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

  const COUNT_WORDS: Record<number, string> = { 2: "Two", 3: "Three", 4: "Four", 5: "Five" };
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
  <p class="kicker">Agents on this machine</p>
  <h2 class="title">{loading ? "Looking for your agents…" : readyCount > 1 ? `${COUNT_WORDS[readyCount] ?? readyCount} agents are ready to work.` : readyCount === 1 ? "One agent is ready to work." : "Connect a coding agent."}</h2>
  <p class="lead">Pytxo runs the CLIs you already use, signed in with your own accounts. Every ready agent can share a job.</p>

  {#if loading}
    <div class="agents" aria-hidden="true">{#each [0, 1, 2, 3, 4] as row}<div class="agent skeleton" style={`--row:${row}`}><i></i><span><i></i><i></i></span></div>{/each}</div>
  {:else}
    <div class="agents" aria-label="Detected agent CLIs">
      {#each usefulAgents as agent (agent.id)}
        <div class="agent">
          <AdeIdentity id={agent.id} />
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
      <p class="summary" role="status">{readyCount} of {usefulAgents.length} ready · {installedCount} installed. You can add more later in Setup.</p>
    {/if}
    {#if readyCount === 0 && !error}<p class="summary">Install an agent or finish sign-in, then check again. You can also finish Desktop setup and connect an agent later.</p>{/if}
  {/if}

  {#if message}<p class="message" role="status">{message}</p>{/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}

  {#snippet actions()}
    {#if readyCount > 0}<Button disabled={loading || openingId !== null} onclick={onContinue}>{readyCount > 1 ? `Continue with ${readyCount} agents` : "Continue"}</Button>
    {:else}<Button disabled={loading || openingId !== null} onclick={onSkip}>Set up agents later</Button>{/if}
    <Button variant="outline" disabled={loading || openingId !== null} onclick={() => void refresh()}>
      {loading ? "Checking…" : "Check again"}
    </Button>
    {#if onRuntimeTools}<Button variant="ghost" onclick={onRuntimeTools}>Terminal tools</Button>{/if}
  {/snippet}
</SetupStepFrame>

<style>
  .install-guide { display: inline-flex; align-items: center; min-height: 40px; padding: 0 10px; border: 1px solid var(--border); border-radius: 6px; color: var(--foreground); text-decoration: none; font-size: 13px; }
  .install-guide:hover { background: var(--accent); }
  .install-guide:focus-visible { outline: 2px solid var(--ring); outline-offset: 2px; }
  .agents { display: grid; width: 100%; border: 1px solid var(--border); border-radius: 10px; background: color-mix(in oklab, var(--card) 70%, transparent); overflow: hidden; text-align: left; }
  .agent { display: flex; align-items: center; gap: 14px; min-height: 56px; padding: 8px 16px; border-bottom: 1px solid var(--border); }
  .agent:last-child { border-bottom: 0; }
  .agent > span { flex: 1; display: grid; gap: 3px; min-width: 0; }
  .agent strong { font-size: 14px; font-weight: 500; }
  .agent small, .summary, .message, .error { color: var(--muted-foreground); font-size: 12px; }
  .agent small { font-family: var(--font-mono, "IBM Plex Mono", monospace); }
  .agent b { color: var(--muted-foreground); font: 500 12px var(--font-mono, "IBM Plex Mono", monospace); }
  .agent b.ready::before { content: ""; display: inline-block; width: 6px; height: 6px; margin-right: 7px; border-radius: 50%; background: currentColor; vertical-align: 1px; }
  .agent b.ready { color: var(--state-verified); }
  .skeleton i { display: block; border-radius: 6px; background: color-mix(in oklab, var(--foreground) 7%, transparent); animation: skeleton 1.2s ease-in-out calc(var(--row) * 90ms) infinite alternate; }
  .skeleton > i { width: 26px; height: 26px; }
  .skeleton span i { width: 40%; height: 10px; }
  .skeleton span i + i { width: 24%; height: 8px; }
  @keyframes skeleton { to { opacity: .35; } }
  @media (prefers-reduced-motion: reduce) { .skeleton i { animation: none; } }
  .message {
    color: var(--primary);
  }
  .summary,
  .message,
  .error {
    margin: 0;
  }
  .error {
    color: var(--destructive, #f87171);
  }
</style>
