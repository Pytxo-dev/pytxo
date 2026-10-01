<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { ipc, onAuthChanged } from "../../lib/ipc";
  import { DECK_THEMES, type DeckTheme } from "../../lib/theme";
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { DropdownMenu, DropdownMenuItem } from "$lib/components/ui/dropdown-menu";

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

  let themeOpen = $state(false);

  const walletUsd = $derived(
    walletMicrocredits != null ? (walletMicrocredits / 1_000_000).toFixed(2) : null,
  );

  async function signOut() {
    await ipc.authClearSession();
    onAuthChange?.();
  }

  let authUnlisten: (() => void) | null = null;

  onMount(async () => {
    authUnlisten = await onAuthChanged(() => {
      onAuthChange?.();
    });
  });

  onDestroy(() => {
    authUnlisten?.();
  });
</script>

<div class="deck-toolbar">
  <div class="deck-toolbar__status">
    <Badge variant="muted" title="{maxAgents >= 64 ? 'Unlimited' : maxAgents} agents max">{tier}</Badge>
    {#if permissionCeiling}
      <Badge variant="outline" title="Org permission ceiling">{permissionCeiling}</Badge>
    {/if}
    {#if walletUsd != null && tier === "ultra"}
      <Badge variant="outline">${walletUsd}</Badge>
    {/if}
    {#if cloudRunBadge === "cloud"}
      <Badge>Cloud</Badge>
    {:else if cloudRunBadge === "fallback"}
      <Badge variant="secondary">Local</Badge>
    {/if}
    {#if cliMissing}
      <Badge variant="outline" title="CLI not on PATH">CLI missing</Badge>
    {/if}
  </div>

  <div class="deck-toolbar__actions">
    {#if subscriptionPortalUrl}
      <Button
        variant="ghost"
        size="sm"
        onclick={() => window.open(subscriptionPortalUrl!, "_blank", "noopener,noreferrer")}
      >
        Plans
      </Button>
    {/if}
    <DropdownMenu bind:open={themeOpen} align="end">
      {#snippet trigger({ toggle })}
        <Button variant="outline" size="sm" onclick={toggle}>Theme</Button>
      {/snippet}
      {#each DECK_THEMES as t}
        <DropdownMenuItem
          selected={theme === t.id}
          onclick={() => {
            theme = t.id;
            themeOpen = false;
          }}
        >
          {t.label}
        </DropdownMenuItem>
      {/each}
    </DropdownMenu>
    <Button variant="outline" size="sm" onclick={onRefresh}>Refresh</Button>
    {#if signedIn}
      <Button variant="ghost" size="sm" onclick={signOut}>Sign out</Button>
    {:else}
      <Badge variant="outline" title="Connect experimental Routing in Settings; general account sign-in is paused">Local Core</Badge>
    {/if}
  </div>
</div>

<style>
  .deck-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
    background: var(--background);
    flex-shrink: 0;
  }
  .deck-toolbar__status,
  .deck-toolbar__actions {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    flex-wrap: wrap;
  }
</style>
