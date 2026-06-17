<script lang="ts">
  import { ipc } from "../../lib/ipc";

  let {
    cmd = $bindable(""),
    dispatchRepo = $bindable(""),
    tier = "core",
    maxAgents = 3,
    signedIn = false,
    onDryRun,
    onDispatch,
    onStop,
    onRefresh,
    onAuthChange,
  }: {
    cmd?: string;
    dispatchRepo?: string;
    tier?: string;
    maxAgents?: number;
    signedIn?: boolean;
    onDryRun: () => void;
    onDispatch: () => void;
    onStop: () => void;
    onRefresh: () => void;
    onAuthChange?: () => void;
  } = $props();

  async function signIn() {
    await ipc.authOpenSignIn();
  }

  async function signOut() {
    await ipc.authClearSession();
    onAuthChange?.();
  }
</script>

<header class="header glass-panel chroma-edge-top">
  <div class="header__brand">
    <h1 class="chroma-text">Pytxo Reality Deck</h1>
    <span class="tier-badge tabular-nums">
      {tier} · {maxAgents >= 64 ? "∞" : maxAgents} agents
      {#if tier === "core"}
        · <a class="upgrade" href="https://pytxo.com/plans" target="_blank" rel="noopener noreferrer">Upgrade</a>
      {/if}
    </span>
  </div>
  <div class="header__actions">
    <input
      bind:value={dispatchRepo}
      placeholder="repo path (blank = selected domain)"
      title="Absolute path to a project folder"
    />
    <input bind:value={cmd} placeholder="command" />
    <button onclick={onDryRun}>Dry run</button>
    <button class="primary chroma-glow" onclick={onDispatch}>Dispatch</button>
    <button onclick={onStop}>Stop</button>
    <button onclick={onRefresh}>Refresh</button>
    {#if signedIn}
      <button onclick={signOut}>Sign out</button>
    {:else}
      <button class="primary" onclick={signIn}>Sign in</button>
    {/if}
  </div>
  <div class="header-chroma-line" aria-hidden="true"></div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.75rem 1rem;
    margin: 0.5rem;
    gap: 1rem;
    flex-wrap: wrap;
    position: relative;
  }
  .header__brand {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }
  h1 {
    font-size: 1.1rem;
    margin: 0;
    font-weight: 600;
  }
  .tier-badge {
    font-size: 0.7rem;
    color: var(--muted-foreground);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .upgrade {
    color: var(--brand-teal);
    text-decoration: none;
    text-transform: none;
    letter-spacing: normal;
  }
  .upgrade:hover {
    text-decoration: underline;
  }
  .header__actions {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    align-items: center;
  }
  .header-chroma-line {
    position: absolute;
    bottom: 0;
    left: 1rem;
    right: 1rem;
  }
</style>
