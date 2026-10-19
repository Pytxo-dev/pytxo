<script lang="ts">
  import AdeIdentity from "./AdeIdentity.svelte";
  import { onMount, tick } from "svelte";
  import IconAlertCircle from "@tabler/icons-svelte/icons/alert-circle";
  import IconCircleCheck from "@tabler/icons-svelte/icons/circle-check";
  import IconCloud from "@tabler/icons-svelte/icons/cloud";
  import IconExternalLink from "@tabler/icons-svelte/icons/external-link";
  import IconLoader2 from "@tabler/icons-svelte/icons/loader-2";
  import IconLogin2 from "@tabler/icons-svelte/icons/login-2";
  import IconPlugConnected from "@tabler/icons-svelte/icons/plug-connected";
  import IconPlus from "@tabler/icons-svelte/icons/plus";
  import IconRefresh from "@tabler/icons-svelte/icons/refresh";
  import IconShieldLock from "@tabler/icons-svelte/icons/shield-lock";
  import { ACCOUNT_STATE_LABEL, DESKTOP_BETA_ADES, isAdeRunnable, isBetaAde } from "../../lib/ade-status";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AdeCliStatusDto, AgentAccountDto } from "../../lib/types";

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

  let accounts = $state<AgentAccountDto[]>([]);
  let accountsFailed = $state(false);
  let accountBusy = $state<string | null>(null);
  let addingFor = $state<string | null>(null);
  let newAccountName = $state("");
  let confirmRemove = $state<string | null>(null);
  const ACCOUNT_NAME = /^[a-z0-9][a-z0-9-]{0,31}$/;
  const newNameValid = $derived(ACCOUNT_NAME.test(newAccountName) && newAccountName !== "default");
  const accountsFor = (cliId: string) => accounts.filter((row) => row.cli === cliId);

  function showError(error: unknown) {
    adeMessage = error instanceof Error ? error.message : String(error);
    adeMessageTone = "error";
  }

  async function loadAccounts() {
    try {
      accounts = await backend.listAgentAccounts();
      accountsFailed = false;
    } catch (error) {
      accounts = [];
      accountsFailed = true;
      showError(error);
    }
  }

  async function refreshAdeClis() {
    adeLoading = true;
    adeMessage = null;
    adeMessageTone = null;
    try {
      const [clis] = await Promise.all([backend.listAdeClis(), loadAccounts()]);
      adeClis = clis;
    } catch (error) {
      adeClis = [];
      showError(error);
    } finally {
      adeLoading = false;
    }
  }

  /** One account action at a time; the list re-probes in place afterwards. */
  async function accountAction(key: string, run: () => Promise<string | void>) {
    if (accountBusy) return;
    accountBusy = key;
    adeMessage = null;
    adeMessageTone = null;
    confirmRemove = null;
    try {
      const message = await run();
      if (message) {
        adeMessage = message;
        adeMessageTone = "success";
      }
      const [clis] = await Promise.all([backend.listAdeClis(), loadAccounts()]);
      adeClis = clis;
    } catch (error) {
      showError(error);
    } finally {
      accountBusy = null;
    }
  }

  async function startAdding(cliId: string) {
    addingFor = cliId;
    newAccountName = "";
    confirmRemove = null;
    await tick();
    document.getElementById(`account-name-${cliId}`)?.focus();
  }

  function addAccount(cli: AdeCliStatusDto) {
    if (!newNameValid) return;
    const name = newAccountName;
    void accountAction(`${cli.id}/+`, async () => {
      const launched = await backend.addAgentAccount(cli.id, name);
      addingFor = null;
      return launched.message;
    });
  }

  async function openAdeLogin(cli: AdeCliStatusDto) {
    if (!cli.installed || !cli.login_supported || loginOpeningId) return;
    loginOpeningId = cli.id;
    adeMessage = null;
    adeMessageTone = null;
    try {
      // Sign in the account new work uses, not always the CLI's own home.
      const launched = await backend.startAdeLogin(cli.id, cli.active_account ?? undefined);
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
    {#if cli.installed && cli.active_account != null}
      {@const rows = accountsFor(cli.id)}
      <div class="accounts" role="group" aria-label={`${cli.display_name} accounts`}>
        <div class="accounts-heading">
          <span>Accounts</span>
          {#if addingFor !== cli.id}
            <button class="quiet" disabled={accountBusy !== null} onclick={() => void startAdding(cli.id)}><IconPlus size={12} /> Add account</button>
          {/if}
        </div>
        <ul>
          {#each rows as row (row.account)}
            {@const key = `${row.cli}/${row.account}`}
            <li>
              <span class="account-name">{row.account === "default" ? "Default" : row.account}</span>
              <span class="account-state" data-state={row.state}>{accountBusy === key ? "Working…" : ACCOUNT_STATE_LABEL[row.state]}</span>
              <span class="account-actions">
                {#if row.active}
                  <span class="in-use" title="New work on this CLI signs in with this account">Selected</span>
                {:else}
                  <button class="quiet" disabled={accountBusy !== null} aria-label={`Select ${row.account === "default" ? "Default" : row.account} for new work`} onclick={() => void accountAction(key, () => backend.useAgentAccount(row.cli, row.account))}>Select</button>
                {/if}
                {#if row.state === "signed_out" || row.state === "unknown"}
                  <button class="quiet" disabled={accountBusy !== null} onclick={() => void accountAction(key, async () => (await backend.startAdeLogin(row.cli, row.account)).message)}><IconLogin2 size={12} /> Sign in</button>
                {/if}
                {#if row.account !== "default"}
                  {#if confirmRemove === key}
                    <button class="quiet danger" disabled={accountBusy !== null} onclick={() => void accountAction(key, async () => { await backend.removeAgentAccount(row.cli, row.account); return `Signed out and removed ${row.account}.`; })}>Sign out and remove</button>
                    <button class="quiet" onclick={() => (confirmRemove = null)}>Keep</button>
                  {:else}
                    <button class="quiet" disabled={accountBusy !== null} onclick={() => (confirmRemove = key)}>Remove</button>
                  {/if}
                {/if}
              </span>
            </li>
          {:else}
            <li><span class="account-name">Default</span><span class="account-state" data-state="unknown">{accountsFailed ? "Could not check accounts" : "Checking…"}</span><span></span></li>
          {/each}
        </ul>
        {#if addingFor === cli.id}
          <form class="account-add" onsubmit={(event) => { event.preventDefault(); addAccount(cli); }}>
            <label for={`account-name-${cli.id}`}>New account name</label>
            <input id={`account-name-${cli.id}`} bind:value={newAccountName} placeholder="work" maxlength="32" autocomplete="off" spellcheck="false" aria-invalid={newAccountName !== "" && !newNameValid} aria-describedby={`account-hint-${cli.id}`} onkeydown={(event) => { if (event.key === "Escape") addingFor = null; }} />
            <button class="quiet" type="submit" disabled={!newNameValid || accountBusy !== null}>{#if accountBusy === `${cli.id}/+`}<IconLoader2 size={12} class="spin" />{/if}Add and sign in</button>
            <button class="quiet" type="button" onclick={() => (addingFor = null)}>Cancel</button>
            <small id={`account-hint-${cli.id}`}>Lowercase letters, digits and dashes. The account gets its own {cli.display_name} folder and signs in through {cli.auth_owner}.</small>
          </form>
        {/if}
      </div>
    {/if}
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
  /* Accounts sit inside their CLI row so the row's separator still frames them. */
  .accounts { grid-column: 1 / -1; display: grid; gap: 2px; margin-top: 6px; padding-top: 6px; border-top: 1px solid var(--pytxo-line-soft); }
  .accounts-heading { display: flex; align-items: center; justify-content: space-between; min-height: 32px; color: var(--pytxo-text-muted); font-size: 11px; font-weight: 600; }
  .accounts ul { margin: 0; padding: 0; list-style: none; }
  .accounts li { display: grid; grid-template-columns: minmax(150px, .9fr) minmax(0, 1.5fr) auto; align-items: center; gap: 4px 24px; min-height: 40px; border-top: 1px solid var(--pytxo-line-soft); }
  .account-name { min-width: 0; overflow-wrap: anywhere; font: 12px var(--pytxo-font-mono), monospace; color: var(--pytxo-text-strong); }
  .account-state { font-size: 12px; color: var(--pytxo-text-soft); }
  .account-state[data-state="signed_in"]::before { content: ""; display: inline-block; width: 6px; height: 6px; margin-right: 7px; border-radius: 50%; background: var(--state-verified); vertical-align: 1px; }
  .account-state[data-state="signed_out"] { color: var(--state-attention); }
  .account-state[data-state="unknown"], .account-state[data-state="not_installed"] { color: var(--pytxo-text-muted); }
  .account-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; align-items: center; gap: 6px 8px; }
  .account-actions .in-use { color: var(--pytxo-text-muted); font-size: 11px; padding: 0 4px; }
  .screen .accounts .quiet { display: inline-flex; align-items: center; justify-content: center; gap: 6px; height: auto; min-height: 32px; padding: 6px 10px; }
  .screen .accounts .quiet.danger { border-color: var(--state-refuted); color: var(--state-refuted); }
  .account-add { display: grid; grid-template-columns: auto minmax(120px, 220px) auto auto 1fr; align-items: center; gap: 6px 8px; padding: 8px 0 4px; border-top: 1px solid var(--pytxo-line-soft); }
  .account-add label { font-size: 12px; color: var(--pytxo-text-soft); }
  .account-add input { min-height: 32px; padding: 0 9px; border: 1px solid var(--pytxo-line); border-radius: 5px; background: var(--pytxo-surface-raised); color: var(--pytxo-text-strong); font: 12px var(--pytxo-font-mono), monospace; }
  .account-add input:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 1px; }
  .account-add input[aria-invalid="true"] { border-color: var(--state-attention); }
  .account-add small { grid-column: 1 / -1; margin: 0; }
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
    .accounts li { grid-template-columns: minmax(0, 1fr); gap: 4px; padding-block: 6px; }
    .account-actions { justify-content: flex-start; }
    .account-add { grid-template-columns: minmax(0, 1fr); }
    .catalog-heading, .supported-catalog > summary { align-items: flex-start; flex-direction: column; justify-content: center; gap: 3px; padding-block: 8px; }
  }
</style>
