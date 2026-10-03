<script lang="ts">
  import AdeIdentity from "./AdeIdentity.svelte";
  import { onMount } from "svelte";
  import IconAlertCircle from "@tabler/icons-svelte/icons/alert-circle";
  import IconCircleCheck from "@tabler/icons-svelte/icons/circle-check";
  import IconCloud from "@tabler/icons-svelte/icons/cloud";
  import IconExternalLink from "@tabler/icons-svelte/icons/external-link";
  import IconLoader2 from "@tabler/icons-svelte/icons/loader-2";
  import IconLogin2 from "@tabler/icons-svelte/icons/login-2";
  import IconPlugConnected from "@tabler/icons-svelte/icons/plug-connected";
  import IconRefresh from "@tabler/icons-svelte/icons/refresh";
  import IconShieldLock from "@tabler/icons-svelte/icons/shield-lock";
  import { DESKTOP_BETA_ADES, isAdeRunnable, isBetaAde } from "../../lib/ade-status";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AdeCliStatusDto } from "../../lib/types";

  let {
    backend,
    onUseInMission,
    embedded = false,
  }: {
    backend: DesktopBackend;
    onUseInMission: (adeId: string) => void;
    /** Rendered inside Setup, so the surrounding screen owns the heading. */
    embedded?: boolean;
  } = $props();

  let adeClis = $state<AdeCliStatusDto[]>([]);
  let adeLoading = $state(true);
  let adeMessage = $state<string | null>(null);
  let adeMessageTone = $state<"success" | "error" | null>(null);
  let loginOpeningId = $state<string | null>(null);

  const byReadiness = (a: AdeCliStatusDto, b: AdeCliStatusDto) => Number(b.installed) - Number(a.installed) || a.display_name.localeCompare(b.display_name);
  const betaOrder = (cli: AdeCliStatusDto) => (DESKTOP_BETA_ADES as readonly string[]).indexOf(cli.id);
  const betaAgents = $derived(adeClis.filter(cli => isBetaAde(cli.id)).sort((a, b) => Number(b.installed) - Number(a.installed) || betaOrder(a) - betaOrder(b)));
  const additionalAgents = $derived(adeClis.filter(cli => !isBetaAde(cli.id)).sort(byReadiness));

  async function refreshAdeClis() {
    adeLoading = true;
    adeMessage = null;
    adeMessageTone = null;
    try {
      adeClis = await backend.listAdeClis();
    } catch (error) {
      adeClis = [];
      adeMessage = error instanceof Error ? error.message : String(error);
      adeMessageTone = "error";
    } finally {
      adeLoading = false;
    }
  }

  async function openAdeLogin(cli: AdeCliStatusDto) {
    if (!cli.installed || !cli.login_supported || loginOpeningId) return;
    loginOpeningId = cli.id;
    adeMessage = null;
    adeMessageTone = null;
    try {
      const launched = await backend.startAdeLogin(cli.id);
      adeMessage = launched.message;
      adeMessageTone = "success";
    } catch (error) {
      adeMessage = error instanceof Error ? error.message : String(error);
      adeMessageTone = "error";
    } finally {
      loginOpeningId = null;
    }
  }

  onMount(() => {
    void refreshAdeClis();
  });
</script>

{#snippet agentRow(cli: AdeCliStatusDto)}
  <div class="agent-row" role="group" aria-label={cli.display_name}>
    <div>
      <strong class="ade-name"><AdeIdentity id={cli.id} />{cli.display_name}</strong>
      <small class="mono">{cli.default_cmd}</small>
    </div>
    <div class="agent-status">
      <p class="state-line"><strong class="state" data-state={!cli.installed ? "missing" : isAdeRunnable(cli) ? "ready" : "attention"}>{!cli.installed ? "Not installed" : isAdeRunnable(cli) ? "Ready" : "Not ready"}</strong>{#if cli.installed}<span class="auth">{cli.auth_label}</span>{/if}</p>
      <small>{cli.detail}</small>
    </div>
    <div class="row-actions">
      <a href={cli.docs_url} target="_blank" rel="noopener noreferrer">
        {cli.installed ? "Docs" : "Install guide"} <IconExternalLink size={12} />
      </a>
      {#if isAdeRunnable(cli)}
        <button class="quiet" onclick={() => onUseInMission(cli.id)}>Use in new work</button>
      {/if}
      {#if cli.installed && cli.login_supported && cli.login_label && cli.auth_state !== "signed_in"}
        <button class="quiet" disabled={loginOpeningId !== null} onclick={() => void openAdeLogin(cli)}>
          {#if loginOpeningId === cli.id}<IconLoader2 size={12} class="spin" />{:else}<IconLogin2 size={12} />{/if}
          {cli.login_label}
        </button>
      {/if}
    </div>
  </div>
{/snippet}

<section class="screen collection-screen" class:embedded>
  <header class="screen-heading" class:compact={embedded}>
    <div>
      {#if embedded}<h2>Agent CLIs</h2>{:else}<h1>Agents</h1>{/if}
    </div>
    <button class="quiet integration-refresh" onclick={() => void refreshAdeClis()} disabled={adeLoading}>
      <IconRefresh size={14} class={adeLoading ? "spin" : undefined} />
      {adeLoading ? "Checking" : "Recheck all"}
    </button>
  </header>

  <p class="honesty">
    <IconShieldLock size={14} />
    Pytxo never reads token stores; credentials stay with each vendor CLI.
  </p>
  <p class="beta-intro">Use any of these agents alone, or put several on one job. Each signs in with its own account; local work needs no Pytxo account.</p>

  <article class="panel">
    {#if adeMessage}
      <p class="integration-message" class:error={adeMessageTone === "error"} role={adeMessageTone === "error" ? "alert" : "status"}>
        {#if adeMessageTone === "error"}<IconAlertCircle size={14} />{:else}<IconCircleCheck size={14} />{/if}
        {adeMessage}
      </p>
    {/if}
    {#if adeLoading}
      <div class="cli-loading" role="status"><IconLoader2 size={18} class="spin" /><div><strong>Checking installed CLIs and sessions…</strong><span>No model request is sent.</span></div></div>
    {:else}
      <div class="catalog-heading">
        <strong>Beta agents</strong>
        <span>Installed and signed in are checked separately</span>
      </div>
      {#each betaAgents as cli (cli.id)}
        {@render agentRow(cli)}
      {:else}
        <p class="beta-intro">Agent status is unavailable. Recheck the agent catalog to continue setup.</p>
      {/each}
      {#if additionalAgents.length}
        <details class="supported-catalog">
          <summary>
            <span>Additional agents</span>
            <small>{additionalAgents.filter(cli => cli.installed).length} installed · outside this beta’s focus</small>
          </summary>
          <div class="supported-rows">
            <p class="beta-intro">Detected for advanced use. Desktop Beta runs only the Beta agents above.</p>
            {#each additionalAgents as cli (cli.id)}
              {@render agentRow(cli)}
            {/each}
          </div>
        </details>
      {/if}
    {/if}
  </article>

  <details class="advanced-integrations">
    <summary>Editor and cloud integrations</summary>
  <article class="panel integration-options">
    <div class="integration-option">
      <div>
        <strong><IconPlugConnected size={16} /> MCP hub</strong>
        <small>Configure in your IDE via the <code class="mono">pytxo-mcp</code> stdio server.</small>
      </div>
      <div class="integration-command">
        <small class="mono">npx pytxo-mcp</small>
      </div>
    </div>
    <div class="integration-option">
      <div>
        <strong><IconCloud size={16} /> Cloud sandboxes</strong>
        <small>Local execution only until a cloud dispatcher is configured.</small>
      </div>
    </div>
  </article>
  </details>
</section>

<style>
  .ade-name { display: flex; align-items: center; gap: 8px; }
  .state-line { margin: 0; font-size: 13px; }
  .state-line .auth { color: var(--pytxo-text-soft); }
  .state-line .auth::before { content: " · "; color: var(--pytxo-text-muted); }
  .agent-status .state[data-state="ready"]::before { content: ""; display: inline-block; width: 6px; height: 6px; margin-right: 7px; border-radius: 50%; background: var(--state-verified); vertical-align: 2px; }
  .agent-status .state[data-state="missing"] { color: var(--pytxo-text-muted); }
  .agent-status small { display: block; margin-top: 3px; color: var(--pytxo-text-muted); font-size: 12px; }
  .beta-intro { margin: 0; padding: 12px 16px; font-size: 13px; line-height: 1.5; color: var(--pytxo-text-soft); }
  .advanced-integrations > summary { padding: 12px 0; cursor: pointer; font-size: 13px; color: var(--pytxo-text-soft); }
  .cli-loading { display: flex; align-items: center; gap: 12px; padding: 20px; }
  .cli-loading div { display: grid; gap: 4px; }
  .cli-loading strong { font-size: 13px; }
  .cli-loading span { font-size: 12px; color: var(--pytxo-text-muted); }
  .integration-options { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .catalog-heading { display: flex; align-items: center; justify-content: space-between; gap: 16px; min-height: 40px; padding: 0 16px; color: var(--pytxo-text-muted); border-bottom: 1px solid var(--pytxo-line); }
  .catalog-heading strong { color: var(--pytxo-text-strong); font-size: 12px; font-weight: 600; }
  .catalog-heading span { font-size: 11px; font-variant-numeric: tabular-nums; }
  .supported-catalog { border-top: 1px solid var(--pytxo-line); }
  .supported-catalog > summary { display: flex; align-items: center; justify-content: space-between; gap: 16px; min-height: 48px; padding: 0 16px; color: var(--pytxo-text-strong); cursor: pointer; font-size: 12px; font-weight: 600; }
  .supported-catalog > summary small { color: var(--pytxo-text-muted); font-size: 11px; font-weight: 400; }
  .supported-catalog > summary:hover { background: var(--pytxo-surface-hover); }
  .supported-catalog > summary:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: -3px; }
  .supported-rows { border-top: 1px solid var(--pytxo-line); }
  .screen.embedded .panel.integration-options { margin-top: 8px; padding: 0; }
  .integration-option { min-width: 0; padding: 14px 16px; display: flex; align-items: flex-start; gap: 16px; }
  .integration-option > div:first-child { min-width: 0; flex: 1; }
  .integration-command { flex: none; }
  .integration-command small { margin-top: 1px; white-space: nowrap; }
  .integration-option + .integration-option { border-left: 1px solid var(--pytxo-line); }
  .integration-option strong { display: flex; align-items: center; gap: 8px; font-size: 13px; }
  .integration-option small { display: block; margin-top: 6px; font-size: 12px; line-height: 1.5; color: var(--pytxo-text-muted); }
  @media (max-width: 760px) { .integration-options { grid-template-columns: 1fr; } .integration-option + .integration-option { border-left: 0; border-top: 1px solid var(--pytxo-line); } }
  /* Inside Setup the CLI list is section content, not a screen. */
  .screen.embedded {
    padding: 0;
    gap: 12px;
  }
  .screen.embedded .screen-heading h2 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: -0.01em;
  }
  .screen.embedded .agent-row {
    grid-template-columns: minmax(170px, .9fr) minmax(0, 1.5fr) 19rem;
    align-items: center;
    gap: 6px 24px;
    padding: 12px 16px;
  }
  .agent-row > div { min-width: 0; }
  .agent-row small { overflow-wrap: anywhere; line-height: 1.5; }
  .row-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; align-items: center; gap: 8px 12px; }
  .row-actions a { display: inline-flex; align-items: center; gap: 5px; min-height: 40px; color: var(--pytxo-text-muted); font-size: 12px; text-decoration: none; }
  .row-actions a:hover { color: var(--pytxo-text-strong); }
  .row-actions a:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .screen .row-actions .quiet { display: inline-flex; align-items: center; justify-content: center; gap: 6px; height: auto; min-height: 40px; max-width: 100%; padding: 8px 10px; line-height: 1.5; }
  .row-actions :global(svg) { flex: none; }
  /* Narrow Setup panes: actions move under the agent instead of off the edge. */
  @media (max-width: 1180px) {
    .screen.embedded .agent-row { grid-template-columns: minmax(150px, .9fr) minmax(0, 1.5fr); }
    .screen.embedded .agent-row .row-actions { grid-column: 1 / -1; justify-content: flex-start; }
  }
  @media (max-width: 580px) {
    .integration-option { flex-direction: column; gap: 8px; }
    .screen.embedded .agent-row { grid-template-columns: minmax(0, 1fr); }
    .row-actions { justify-content: flex-start; }
    .catalog-heading, .supported-catalog > summary { align-items: flex-start; flex-direction: column; justify-content: center; gap: 3px; padding-block: 8px; }
  }
</style>
