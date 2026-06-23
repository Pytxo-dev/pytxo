<script lang="ts">
  import { ipc } from "../../lib/ipc";

  let {
    cmd = $bindable(""),
    dispatchRepo = $bindable(""),
    tier = "core",
    maxAgents = 3,
    signedIn = false,
    cloudRunBadge = null,
    walletMicrocredits = null as number | null,
    permissionCeiling = null as string | null,
    subscriptionPortalUrl = null as string | null,
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
    cloudRunBadge?: "cloud" | "fallback" | null;
    walletMicrocredits?: number | null;
    permissionCeiling?: string | null;
    subscriptionPortalUrl?: string | null;
    onDryRun: () => void;
    onDispatch: () => void;
    onStop: () => void;
    onRefresh: () => void;
    onAuthChange?: () => void;
  } = $props();

  const walletUsd = $derived(
    walletMicrocredits != null ? (walletMicrocredits / 1_000_000).toFixed(2) : null,
  );

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
      {#if walletUsd != null && tier === "ultra"}
        · <span class="wallet-badge">${walletUsd} credits</span>
      {/if}
      {#if permissionCeiling}
        · <span class="ceiling-badge" title="Org policy ceiling">{permissionCeiling} cap</span>
      {/if}
      {#if cloudRunBadge === "cloud"}
        · <span class="cloud-badge cloud-badge--active">Cloud</span>
      {:else if cloudRunBadge === "fallback"}
        · <span class="cloud-badge cloud-badge--fallback">Local fallback</span>
      {/if}
      {#if tier === "core"}
        · <a class="upgrade" href="https://pytxo.com/plans" target="_blank" rel="noopener noreferrer">Upgrade</a>
      {:else if subscriptionPortalUrl}
        · <a class="upgrade" href={subscriptionPortalUrl} target="_blank" rel="noopener noreferrer">Manage plan</a>
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
  .wallet-badge {
    color: var(--brand-gold);
    text-transform: none;
    letter-spacing: normal;
  }
  .ceiling-badge {
    color: var(--brand-violet);
    text-transform: uppercase;
    font-size: 0.65rem;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
    border: 1px solid color-mix(in srgb, var(--brand-violet) 35%, transparent);
  }
  .cloud-badge {
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-size: 0.65rem;
    padding: 0.1rem 0.35rem;
    border-radius: 3px;
  }
  .cloud-badge--active {
    color: var(--brand-violet);
    border: 1px solid color-mix(in srgb, var(--brand-violet) 40%, transparent);
  }
  .cloud-badge--fallback {
    color: var(--brand-gold);
    border: 1px solid color-mix(in srgb, var(--brand-gold) 40%, transparent);
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
