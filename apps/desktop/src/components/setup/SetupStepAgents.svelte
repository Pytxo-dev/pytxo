<script lang="ts">
  import { onMount } from "svelte";
  import { createDesktopBackend } from "../../lib/desktop-backend";
  import type { AdeCliStatusDto } from "../../lib/types";
  import { Button } from "$lib/components/ui/button";

  let {
    onContinue,
    onSkip,
  }: {
    onContinue: () => void;
    onSkip: () => void;
  } = $props();

  const backend = createDesktopBackend();
  let agents = $state<AdeCliStatusDto[]>([]);
  let loading = $state(true);
  let openingId = $state<string | null>(null);
  let message = $state("");
  let error = $state("");

  const usefulAgents = $derived(
    [...agents]
      .sort((a, b) => {
        const score = (agent: AdeCliStatusDto) =>
          Number(agent.installed) * 4 +
          Number(agent.auth_state === "signed_in") * 2 +
          Number(agent.login_supported);
        return score(b) - score(a);
      })
      .slice(0, 5),
  );
  const installedCount = $derived(agents.filter((agent) => agent.installed).length);

  async function refresh() {
    loading = true;
    error = "";
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

<div class="step">
  <h2 class="title">Connect your coding agents</h2>
  <p class="lead">
    Keep the CLIs you already use. Pytxo checks non-secret session status and opens each vendor's
    official sign-in; credentials never move into Pytxo.
  </p>

  {#if loading}
    <p class="status">Checking installed agent CLIs…</p>
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
          {:else}
            <b class:ready={agent.auth_state === "signed_in"}>
              {agent.auth_state === "signed_in" ? "Ready" : agent.installed ? "Installed" : "Later"}
            </b>
          {/if}
        </div>
      {/each}
    </div>
    <p class="summary">{installedCount} agent CLI{installedCount === 1 ? "" : "s"} detected. You can recheck and add more from Integrations.</p>
  {/if}

  {#if message}<p class="message" role="status">{message}</p>{/if}
  {#if error}<p class="error" role="alert">{error}</p>{/if}

  <div class="actions">
    <Button onclick={onContinue}>Continue</Button>
    <Button variant="ghost" onclick={onSkip}>Skip for now</Button>
  </div>
</div>

<style>
  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: 100%;
    max-width: 560px;
    margin: 0 auto;
    gap: 0.85rem;
    text-align: center;
  }
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
    font-size: 0.68rem;
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
  .actions {
    display: grid;
    gap: 0.45rem;
    width: 220px;
  }
</style>
