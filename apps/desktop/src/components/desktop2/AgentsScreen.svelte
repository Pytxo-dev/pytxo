<script lang="ts">
  import { onMount } from "svelte";
  import {
    IconAlertCircle,
    IconCircleCheck,
    IconCloud,
    IconExternalLink,
    IconLoader2,
    IconLogin2,
    IconPlugConnected,
    IconRefresh,
    IconShieldLock,
    IconTerminal2,
  } from "@tabler/icons-svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AdeCliStatusDto } from "../../lib/types";

  let {
    backend,
    onUseInMission,
  }: {
    backend: DesktopBackend;
    onUseInMission: () => void;
  } = $props();

  let adeClis = $state<AdeCliStatusDto[]>([]);
  let adeLoading = $state(true);
  let adeMessage = $state<string | null>(null);
  let adeMessageTone = $state<"success" | "error" | null>(null);
  let loginOpeningId = $state<string | null>(null);

  const sortedAdeClis = $derived(
    [...adeClis].sort((a, b) => {
      const priority = ["codex", "claude", "cursor", "opencode", "gemini", "copilot", "aider", "agy"];
      return priority.indexOf(a.id) - priority.indexOf(b.id);
    }),
  );

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

<section class="screen collection-screen">
  <header class="screen-heading">
    <div>
      <h1>Agents</h1>
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

  <article class="panel">
    {#if adeMessage}
      <p class="integration-message" class:error={adeMessageTone === "error"} role={adeMessageTone === "error" ? "alert" : "status"}>
        {#if adeMessageTone === "error"}<IconAlertCircle size={14} />{:else}<IconCircleCheck size={14} />{/if}
        {adeMessage}
      </p>
    {/if}
    {#if adeLoading}
      <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Checking installed CLIs and sessions…</strong><span>No model request is sent.</span></div>
    {:else}
      {#each sortedAdeClis as cli (cli.id)}
        <div class="agent-row">
          <div>
            <strong>{cli.display_name}</strong>
            <small class="mono">{cli.default_cmd}</small>
          </div>
          <div>
            <strong>{cli.installed ? "Installed" : "Not installed"}</strong>
            <small>{cli.detail}</small>
          </div>
          <div>
            <strong>{cli.auth_label}</strong>
            <small>Owned by {cli.auth_owner}</small>
          </div>
          <div class="row-actions">
            <a href={cli.docs_url} target="_blank" rel="noopener noreferrer">
              {cli.installed ? "Docs" : "Install guide"} <IconExternalLink size={12} />
            </a>
            {#if cli.installed && cli.auth_state === "signed_in"}
              <button class="quiet" onclick={onUseInMission}>Use in mission</button>
            {:else if cli.installed && cli.login_supported && cli.login_label}
              <button class="quiet" disabled={loginOpeningId !== null} onclick={() => void openAdeLogin(cli)}>
                {#if loginOpeningId === cli.id}<IconLoader2 size={12} class="spin" />{:else}<IconLogin2 size={12} />{/if}
                {cli.login_label}
              </button>
            {/if}
          </div>
        </div>
      {/each}
    {/if}
  </article>

  <article class="panel">
    <div class="agent-row">
      <div>
        <strong><IconPlugConnected size={16} /> MCP hub</strong>
        <small>Configure in your IDE via the <code class="mono">pytxo-mcp</code> stdio server.</small>
      </div>
      <div></div>
      <div>
        <small class="mono">npx pytxo-mcp</small>
      </div>
      <div></div>
    </div>
    <div class="agent-row">
      <div>
        <strong><IconCloud size={16} /> Cloud sandboxes</strong>
        <small>Local execution only until a cloud dispatcher is configured.</small>
      </div>
      <div></div>
      <div></div>
      <div></div>
    </div>
  </article>
</section>
