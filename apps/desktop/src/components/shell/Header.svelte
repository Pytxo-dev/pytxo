<script lang="ts">
  import { ipc } from "../../lib/ipc";
  import ThemePicker from "./ThemePicker.svelte";
  import type { DeckTheme } from "../../lib/theme";

  let {
    theme = $bindable("void" as DeckTheme),
    tier = "core",
    maxAgents = 3,
    signedIn = false,
    cliMissing = false,
    cloudRunBadge = null,
    walletMicrocredits = null as number | null,
    permissionCeiling = null as string | null,
    subscriptionPortalUrl = null as string | null,
    onRefresh,
    onAuthChange,
  }: {
    theme?: DeckTheme;
    tier?: string;
    maxAgents?: number;
    signedIn?: boolean;
    cliMissing?: boolean;
    cloudRunBadge?: "cloud" | "fallback" | null;
    walletMicrocredits?: number | null;
    permissionCeiling?: string | null;
    subscriptionPortalUrl?: string | null;
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

<header class="header glass-panel">
  <div class="header__brand">
    <img src="/logo.png" alt="" width="28" height="28" class="header__logo" />
    <div>
      <h1 class="chroma-text">Reality Deck</h1>
      <div class="badges tabular-nums">
        <span class="pill">{tier}</span>
        <span class="pill">{maxAgents >= 64 ? "∞" : maxAgents} agents</span>
        {#if walletUsd != null && tier === "ultra"}
          <span class="pill pill--gold">${walletUsd}</span>
        {/if}
        {#if permissionCeiling}
          <span class="pill pill--violet">{permissionCeiling}</span>
        {/if}
        {#if cloudRunBadge === "cloud"}
          <span class="pill pill--teal">Cloud</span>
        {:else if cloudRunBadge === "fallback"}
          <span class="pill pill--gold">Local</span>
        {/if}
        {#if cliMissing}
          <span class="pill pill--warn" title="CLI not on PATH">CLI</span>
        {/if}
      </div>
    </div>
  </div>

  <div class="header__actions">
    <ThemePicker bind:theme />
    <button type="button" onclick={onRefresh}>Refresh</button>
    {#if tier === "core"}
      <a class="link" href="https://pytxo.com/plans" target="_blank" rel="noopener noreferrer">
        Upgrade
      </a>
    {:else if subscriptionPortalUrl}
      <a class="link" href={subscriptionPortalUrl} target="_blank" rel="noopener noreferrer">
        Plan
      </a>
    {/if}
    {#if signedIn}
      <button type="button" onclick={signOut}>Sign out</button>
    {:else}
      <button type="button" class="primary" onclick={signIn}>Sign in</button>
    {/if}
  </div>
</header>

<style>
  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.65rem 1rem;
    margin: 0.5rem 0.5rem 0;
    gap: 1rem;
    flex-wrap: wrap;
    border-radius: 16px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
  }
  .header__brand {
    display: flex;
    align-items: center;
    gap: 0.65rem;
  }
  .header__logo {
    border-radius: 8px;
    outline: 1px solid rgba(255, 255, 255, 0.08);
  }
  h1 {
    font-size: 1rem;
    margin: 0;
    font-weight: 600;
    text-wrap: balance;
  }
  .badges {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
    margin-top: 0.2rem;
  }
  .pill {
    font-size: 0.62rem;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    padding: 0.12rem 0.4rem;
    border-radius: 6px;
    color: var(--muted-foreground);
    background: color-mix(in oklab, var(--foreground) 8%, transparent);
  }
  .pill--gold {
    color: var(--brand-gold);
    background: color-mix(in oklab, var(--brand-gold) 12%, transparent);
  }
  .pill--violet {
    color: var(--brand-violet);
    background: color-mix(in oklab, var(--brand-violet) 12%, transparent);
  }
  .pill--teal {
    color: var(--brand-teal);
    background: color-mix(in oklab, var(--brand-teal) 12%, transparent);
  }
  .pill--warn {
    color: var(--brand-gold);
    border: 1px dashed color-mix(in oklab, var(--brand-gold) 40%, transparent);
  }
  .header__actions {
    display: flex;
    gap: 0.45rem;
    flex-wrap: wrap;
    align-items: center;
  }
  .header__actions button {
    min-height: 40px;
    border-radius: 10px;
    transition: transform 0.15s cubic-bezier(0.2, 0, 0, 1);
  }
  .header__actions button:active {
    transform: scale(0.96);
  }
  .link {
    font-size: 0.85rem;
    color: var(--brand-teal);
    text-decoration: none;
    align-self: center;
    min-height: 40px;
    display: flex;
    align-items: center;
  }
</style>
