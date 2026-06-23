<script lang="ts">
  let {
    tier = "core",
    maxAgents = 3,
    walletMicrocredits = null as number | null,
    cloudEnabled = false,
    permissionCeiling = null as string | null,
    subscriptionPortalUrl = null as string | null,
  }: {
    tier?: string;
    maxAgents?: number;
    walletMicrocredits?: number | null;
    cloudEnabled?: boolean;
    permissionCeiling?: string | null;
    subscriptionPortalUrl?: string | null;
  } = $props();

  const walletUsd = $derived(
    walletMicrocredits != null ? (walletMicrocredits / 1_000_000).toFixed(2) : null,
  );
</script>

<section class="usage-panel">
  <h2 class="panel-title">Usage & billing</h2>
  <dl class="usage-grid">
    <dt>Plan</dt>
    <dd class="capitalize">{tier}</dd>
    <dt>Agent cap</dt>
    <dd>{maxAgents >= 64 ? "∞" : maxAgents}</dd>
    {#if walletUsd != null}
      <dt>Wallet</dt>
      <dd class="tabular-nums">${walletUsd}</dd>
    {/if}
    <dt>Cloud</dt>
    <dd>{cloudEnabled ? "enabled" : "off"}</dd>
    {#if permissionCeiling}
      <dt>Org ceiling</dt>
      <dd class="capitalize">{permissionCeiling}</dd>
    {/if}
  </dl>
  {#if subscriptionPortalUrl}
    <a class="portal-link" href={subscriptionPortalUrl} target="_blank" rel="noopener noreferrer">
      Open subscription portal →
    </a>
  {/if}
</section>

<style>
  .usage-panel {
    padding: 0.75rem 1rem;
    border-top: 1px solid color-mix(in srgb, var(--border) 60%, transparent);
  }
  .panel-title {
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--muted-foreground);
    margin: 0 0 0.5rem;
  }
  .usage-grid {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 0.25rem 0.75rem;
    font-size: 0.8rem;
    margin: 0;
  }
  .usage-grid dt {
    color: var(--muted-foreground);
  }
  .usage-grid dd {
    margin: 0;
    font-weight: 500;
  }
  .portal-link {
    display: inline-block;
    margin-top: 0.5rem;
    font-size: 0.75rem;
    color: var(--brand-teal);
    text-decoration: none;
  }
  .portal-link:hover {
    text-decoration: underline;
  }
</style>
