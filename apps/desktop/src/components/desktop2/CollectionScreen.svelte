<script lang="ts">
  import { onMount } from "svelte";
  import { IconCheck, IconChevronRight, IconCloud, IconFolder, IconGitBranch, IconLoader2, IconPlugConnected, IconPlus, IconSearch, IconSettings, IconShieldLock, IconTerminal2 } from "@tabler/icons-svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { AppRoute } from "../../lib/navigation.svelte";
  import { applyDeckTheme } from "../../lib/theme";
  import { setReducedMotion, setUiDensity, setUiScale, UI_SCALES, uiPrefs } from "../../lib/ui-prefs.svelte";
  import type { AdeCliStatusDto, HitlDto } from "../../lib/types";

  const VOICE_CAPTURE_KEY = "pytxo-desktop-voice-capture-v1";
  const VOICE_CONSENT_KEY = "pytxo-desktop-voice-cloud-consent-v1";
  const SETTINGS_SECTIONS = ["Appearance", "Voice", "Privacy", "Account & billing"] as const;

  let {
    route,
    snapshot,
    backend,
    onRoute,
    tier,
    signedIn,
    cliMissing,
    subscriptionPortalUrl,
    onReplayOnboarding,
    onReviewRun,
    onViewTopology,
    onSelectDomain,
    onWorkspaceOpened,
    onDomainForgotten,
    onApprovalsChanged,
  }: {
    route: AppRoute;
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    onRoute: (route: AppRoute) => void;
    tier: string;
    signedIn: boolean;
    cliMissing: boolean;
    subscriptionPortalUrl: string | null;
    onReplayOnboarding: () => void;
    onReviewRun: (runId: string) => void;
    onViewTopology: (domainId: string) => void;
    onSelectDomain: (domainId: string, opts?: { route?: AppRoute }) => void;
    onWorkspaceOpened: (openedPath?: string | null) => void | Promise<void>;
    onDomainForgotten: (domainId: string) => void;
    onApprovalsChanged: () => void | Promise<void>;
  } = $props();

  let query = $state("");
  let healthFilter = $state<"all" | "active">("all");
  let selectedApprovalId = $state<string | null>(null);
  let selectedSetting = $state<(typeof SETTINGS_SECTIONS)[number]>("Appearance");
  let voiceModelPath = $state<string | null>(null);
  let voiceInstalling = $state(false);
  let showCloudConsent = $state(false);
  let voiceCapture = $state<"both" | "hold" | "click">("both");
  let voiceConsentNoted = $state(false);
  let openedWorkspace = $state<string | null>(null);
  let adeClis = $state<AdeCliStatusDto[]>([]);
  let adeLoading = $state(true);
  let showCleanup = $state(false);
  let forgettingId = $state<string | null>(null);
  let forgetError = $state<string | null>(null);
  let deciding = $state(false);
  let decisionMessage = $state<string | null>(null);

  const availableDomains = $derived(snapshot.domains.filter((d) => d.is_available && !d.is_temporary));
  const staleDomains = $derived(snapshot.domains.filter((d) => !d.is_available || d.is_temporary));
  const filteredDomains = $derived(
    availableDomains
      .filter((d) => d.repo_root.toLowerCase().includes(query.toLowerCase()))
      .filter((d) => (healthFilter === "active" ? d.active_runs > 0 : true)),
  );
  const openApprovals = $derived(snapshot.approvals);
  const selectedApproval: HitlDto | null = $derived(
    openApprovals.find((a) => a.id === selectedApprovalId) ?? openApprovals[0] ?? null,
  );

  function profileHint(domainId: string, repoRoot: string): string {
    const run = snapshot.runs.find((r) => r.repo_root === repoRoot);
    return run?.permission_profile ?? "—";
  }

  async function resolveApproval(approve: boolean) {
    if (!selectedApproval || deciding) return;
    deciding = true;
    decisionMessage = null;
    try {
      if (approve) await backend.approve(selectedApproval.id, selectedApproval.domain_id ?? null);
      else await backend.deny(selectedApproval.id, selectedApproval.domain_id ?? null);
      decisionMessage = approve
        ? "Approved. Blast Shield flush proceeds under the domain permission profile."
        : "Denied. Isolated changes were not flushed.";
      selectedApprovalId = null;
      await onApprovalsChanged();
    } finally {
      deciding = false;
    }
  }

  async function installVoiceModel() {
    voiceInstalling = true;
    try {
      voiceModelPath = await backend.installVoiceModel();
    } finally {
      voiceInstalling = false;
    }
  }

  async function openWorkspace() {
    openedWorkspace = await backend.openWorkspace();
    if (openedWorkspace) await onWorkspaceOpened(openedWorkspace);
  }

  async function forgetDomain(domainId: string) {
    forgettingId = domainId;
    forgetError = null;
    try {
      await backend.forgetDomain(domainId);
      onDomainForgotten(domainId);
    } catch (e) {
      forgetError = e instanceof Error ? e.message : String(e);
    } finally {
      forgettingId = null;
    }
  }

  function setVoiceCapture(value: "both" | "hold" | "click") {
    voiceCapture = value;
    if (typeof localStorage !== "undefined") localStorage.setItem(VOICE_CAPTURE_KEY, value);
  }

  function consentCloudFallback() {
    voiceConsentNoted = true;
    if (typeof localStorage !== "undefined") localStorage.setItem(VOICE_CONSENT_KEY, "true");
    showCloudConsent = false;
  }

  $effect(() => {
    if (route === "approvals" && openApprovals.length && !openApprovals.some((a) => a.id === selectedApprovalId)) {
      selectedApprovalId = openApprovals[0]?.id ?? null;
    }
  });

  onMount(async () => {
    if (typeof localStorage !== "undefined") {
      const capture = localStorage.getItem(VOICE_CAPTURE_KEY);
      if (capture === "both" || capture === "hold" || capture === "click") voiceCapture = capture;
      voiceConsentNoted = localStorage.getItem(VOICE_CONSENT_KEY) === "true";
    }
    try {
      adeClis = await backend.listAdeClis();
    } finally {
      adeLoading = false;
    }
    try {
      voiceModelPath = await backend.voiceModelStatus();
    } catch {
      /* Voice model status is best-effort; installer button remains available. */
    }
  });
</script>

<section class="screen collection-screen">
  <header class="screen-heading">
    <div><p class="eyebrow">Pytxo Desktop</p><h1>{route === "run-review" ? "Run Review" : route[0].toUpperCase() + route.slice(1)}</h1><p>{route === "workspaces" ? "Execution domains, roots, and health in one catalog." : route === "runs" ? "Every orchestration run, from dispatch through recovery." : route === "approvals" ? "Human decisions with the context needed to act confidently." : route === "integrations" ? "ADE CLIs and how to connect the MCP hub." : "Tune appearance, privacy, Voice, and account."}</p></div>
    {#if route === "workspaces"}<button class="primary" onclick={openWorkspace}><IconPlus size={16} /> Add workspace</button>{/if}
  </header>

  {#if route === "workspaces"}
    <div class="toolbar">
      <label><IconSearch size={16} /><input bind:value={query} placeholder="Search workspaces" aria-label="Search workspaces" /></label>
      <button class="quiet" onclick={() => (healthFilter = healthFilter === "all" ? "active" : "all")}>{healthFilter === "all" ? "Health: all" : "Health: active only"}</button>
    </div>
    {#if openedWorkspace}<p class="workspace-opened">Added {openedWorkspace}</p>{/if}
    {#if filteredDomains.length}
      <div class="catalog-grid">
        {#each filteredDomains as domain}
          <article class="workspace-card">
            <div class="workspace-icon"><IconFolder size={19} /></div>
            <div class="workspace-title"><h2>{domain.repo_root.split(/[\\/]/).pop()}</h2><span class="healthy"><i></i>{domain.status}</span></div>
            <p>{domain.repo_root}</p>
            <div class="workspace-stats"><span><small>Active runs</small><strong>{domain.active_runs}</strong></span><span><small>Approvals</small><strong>{domain.hitl_pending}</strong></span><span><small>Policy</small><strong>{profileHint(domain.domain_id, domain.repo_root)}</strong></span></div>
            <button class="card-link" onclick={() => onViewTopology(domain.domain_id)}>View structure <IconGitBranch size={15} /></button>
            <button class="card-link" onclick={() => onSelectDomain(domain.domain_id, { route: "operations" })}>Open workspace <IconChevronRight size={16} /></button>
          </article>
        {/each}
      </div>
    {:else}
      <div class="empty"><IconFolder size={26} /><strong>No workspaces {query || healthFilter === "active" ? "match this filter" : "yet"}</strong><span>{query || healthFilter === "active" ? "Try clearing the search or health filter." : "Add a workspace to start dispatching runs."}</span></div>
    {/if}
    {#if staleDomains.length}
      <button class="cleanup-toggle" onclick={() => (showCleanup = !showCleanup)} aria-expanded={showCleanup}>
        <IconChevronRight size={12} />
        {showCleanup ? "Hide" : "Show"} {staleDomains.length} stale or missing workspace{staleDomains.length === 1 ? "" : "s"}
      </button>
      {#if showCleanup}
        <div class="cleanup-list">
          {#each staleDomains as domain}
            <div class="cleanup-row">
              <p><strong>{domain.repo_root.split(/[\\/]/).pop()}</strong><small>{domain.repo_root} · {domain.is_temporary ? "temporary test artifact" : "path not found on disk"}</small></p>
              <button class="quiet" disabled={forgettingId === domain.domain_id || domain.active_runs > 0 || domain.hitl_pending > 0} onclick={() => forgetDomain(domain.domain_id)}>
                {forgettingId === domain.domain_id ? "Removing…" : domain.active_runs > 0 ? "Has active runs" : domain.hitl_pending > 0 ? "Has approvals" : "Forget"}
              </button>
            </div>
          {/each}
        </div>
        {#if forgetError}<p class="voice-state-message error">{forgetError}</p>{/if}
      {/if}
    {/if}
  {:else if route === "runs"}
    <article class="panel table-panel">
      <div class="panel-head"><h2>Run history</h2><span>{snapshot.runs.length} run{snapshot.runs.length === 1 ? "" : "s"}</span></div>
      <div class="data-table" role="table">
        <div class="table-row table-header" role="row"><span role="columnheader">Run</span><span role="columnheader">Workspace</span><span role="columnheader">Status</span><span role="columnheader">Policy</span><span role="columnheader">Estimate</span></div>
        {#if snapshot.runs.length}
          {#each snapshot.runs as run}
            <div class="table-row" role="row" tabindex="0" onclick={() => onReviewRun(run.id)} onkeydown={(e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); onReviewRun(run.id); } }}><span class="mono" role="cell">{run.id}</span><span role="cell">{run.repo_root.split(/[\\/]/).pop()}</span><span role="cell"><i class:active={run.status === "running"}></i>{run.status}</span><span role="cell">{run.permission_profile}</span><span role="cell">${(run.estimated_cost_usd ?? 0).toFixed(2)} <IconChevronRight size={15} /></span></div>
          {/each}
        {:else}
          <div class="empty"><strong>No runs yet</strong><span>Dispatch a Flow to populate run history.</span></div>
        {/if}
      </div>
    </article>
  {:else if route === "approvals"}
    <div class="approval-layout">
      <article class="panel inbox">
        <div class="panel-head"><h2>Inbox</h2><span>{openApprovals.length} open</span></div>
        {#if openApprovals.length}
          {#each openApprovals as approval}
            <button class="inbox-row" class:active={selectedApproval?.id === approval.id} onclick={() => (selectedApprovalId = approval.id)}>
              <span class="risk">Review</span>
              <strong>{approval.action}</strong>
              <small>{approval.agent_key} · {new Date(Number(approval.created_at_ms)).toLocaleString()}</small>
              <IconChevronRight size={16} />
            </button>
          {/each}
        {:else}
          <div class="empty"><strong>Inbox clear</strong><span>New HITL requests appear here during Galaxy runs.</span></div>
        {/if}
      </article>
      <article class="panel decision-detail">
        <p class="eyebrow">Decision context</p>
        <h2>{selectedApproval?.action ?? "Inbox clear"}</h2>
        <p>{selectedApproval?.reason ?? "All decisions have been resolved."}</p>
        {#if selectedApproval}
          <div class="diff-summary">
            <span><IconShieldLock size={17} /> Blast Shield</span>
            <strong>Approve to flush · Deny to discard</strong>
            <small>Pytxo enforces the execution-domain permission profile when this workspace is flushed. Open Run Review for file-level evidence.</small>
          </div>
          {#if decisionMessage}<p class="voice-state-message">{decisionMessage}</p>{/if}
          <div class="decision-actions">
            <button class="deny" disabled={deciding} onclick={() => resolveApproval(false)}>Deny</button>
            <button class="primary" disabled={deciding} onclick={() => resolveApproval(true)}><IconCheck size={16} /> Approve & flush</button>
          </div>
        {/if}
      </article>
    </div>
  {:else if route === "integrations"}
    <div class="integration-sections">
      <article class="panel">
        <div class="panel-head"><div><p class="eyebrow">Agent development environments</p><h2>ADE registry</h2></div></div>
        {#if adeLoading}
          <div class="empty"><IconLoader2 size={22} class="spin" /><strong>Checking installed CLIs…</strong></div>
        {:else}
          <div class="integration-grid">
            {#each adeClis as cli}
              <div class="integration-card"><div class="provider-icon"><IconTerminal2 size={19} /></div><p><strong>{cli.display_name}</strong><small class="mono">{cli.default_cmd}</small></p><span class:healthy={cli.installed}>{cli.installed ? "Installed" : "Not installed"}</span></div>
            {/each}
          </div>
        {/if}
      </article>
      <article class="panel integration-truth">
        <div class="integration-card wide">
          <div class="provider-icon"><IconPlugConnected size={20} /></div>
          <div>
            <p><strong>MCP hub</strong><small>Configure in your IDE via the <code class="mono">pytxo-mcp</code> stdio server — not an in-app connection panel.</small></p>
            <p class="setup-hint mono">npx pytxo-mcp · Cursor MCP settings · docs: MCP from Cursor</p>
          </div>
        </div>
        <div class="integration-card wide">
          <div class="provider-icon"><IconCloud size={20} /></div>
          <div>
            <p><strong>Cloud sandboxes</strong><small>Local execution only until a cloud dispatcher is configured. No fake cloud toggle here.</small></p>
          </div>
        </div>
      </article>
    </div>
  {:else}
    <div class="settings-layout">
      <nav aria-label="Settings sections">
        {#each SETTINGS_SECTIONS as item}
          <button class:active={item === selectedSetting} onclick={() => (selectedSetting = item)}><IconSettings size={16} />{item}</button>
        {/each}
      </nav>
      <article class="panel settings-panel">
        <p class="eyebrow">{selectedSetting === "Voice" ? "Local-first capture" : selectedSetting === "Privacy" ? "Sovereign Shield" : "Desktop preferences"}</p>
        <h2>{selectedSetting}</h2>
        {#if selectedSetting === "Appearance"}
          <div class="setting-row"><div><strong>Theme</strong><small>Obsidian is canonical; light mode is token-complete.</small></div><div class="segmented"><button onclick={() => applyDeckTheme("void")}>Dark</button><button onclick={() => applyDeckTheme("light")}>Light</button><button onclick={() => applyDeckTheme(matchMedia("(prefers-color-scheme: light)").matches ? "light" : "void")}>System</button></div></div>
          <div class="setting-row"><div><strong>Scale</strong><small>Zooms the whole webview via Tauri, not just CSS.</small></div><div class="segmented">{#each UI_SCALES as opt (opt.value)}<button class:active={uiPrefs.scale === opt.value} onclick={() => setUiScale(opt.value)}>{opt.label}</button>{/each}</div></div>
          <div class="setting-row"><div><strong>Density</strong><small>Compact fits more per screen; comfortable adds breathing room.</small></div><select aria-label="Density" value={uiPrefs.density} onchange={(e) => setUiDensity(e.currentTarget.value === "comfortable" ? "comfortable" : "compact")}><option value="compact">Compact</option><option value="comfortable">Comfortable</option></select></div>
          <div class="setting-row"><div><strong>Reduced motion</strong><small>Force off all transitions and animations, regardless of OS setting.</small></div><button class="toggle" class:active={uiPrefs.reducedMotion} aria-pressed={uiPrefs.reducedMotion} aria-label="Reduced motion" onclick={() => setReducedMotion(!uiPrefs.reducedMotion)}><i></i></button></div>
        {:else if selectedSetting === "Voice"}
          <div class="setting-row"><div><strong>Local transcription model</strong><small>{voiceModelPath ?? "base.en · 148 MB · not installed"}</small></div><button class="quiet" disabled={voiceInstalling} onclick={installVoiceModel}>{voiceInstalling ? "Installing…" : "Install base.en"}</button></div>
          <div class="setting-row"><div><strong>Capture behavior</strong><small>Click-to-record and press-and-hold; audio is never written to disk.</small></div><select aria-label="Voice capture behavior" value={voiceCapture} onchange={(e) => setVoiceCapture(e.currentTarget.value as "both" | "hold" | "click")}><option value="both">Both</option><option value="hold">Press and hold</option><option value="click">Click to record</option></select></div>
          <div class="setting-row"><div><strong>Cloud fallback</strong><small>{voiceConsentNoted ? "Consent noted for a future session (not used while local-only)." : "Requires explicit consent before any cloud transcription."}</small></div><button class="quiet" onclick={() => (showCloudConsent = true)}>{voiceConsentNoted ? "Review again" : "Review consent"}</button></div>
        {:else if selectedSetting === "Privacy"}
          <div class="setting-row"><div><strong>Raw audio retention</strong><small>Bounded memory only; cleared after completion, failure, or cancellation.</small></div><strong>Never stored</strong></div>
          <div class="setting-row"><div><strong>Transcript sanitization</strong><small>Sovereign Shield runs before cloud planning.</small></div><strong>Enabled</strong></div>
        {:else if selectedSetting === "Account & billing"}
          <div class="setting-row"><div><strong>Account</strong><small>{signedIn ? "Signed in to your Pytxo account." : "Running in local mode; no account linked."}</small></div><strong>{signedIn ? "Signed in" : "Local mode"}</strong></div>
          <div class="setting-row"><div><strong>Tier</strong><small>Controls concurrent agent limits and cloud features.</small></div><strong class="mono">{tier}</strong></div>
          {#if signedIn && subscriptionPortalUrl}
            <div class="setting-row"><div><strong>Billing portal</strong><small>Manage plan, payment method, and invoices.</small></div><a class="quiet" href={subscriptionPortalUrl} target="_blank" rel="noopener noreferrer">Open portal</a></div>
          {/if}
          {#if cliMissing}
            <div class="setting-row"><div><strong>Pytxo CLI</strong><small>Not detected on PATH. Terminal parity and MCP tools need it.</small></div><strong>Missing</strong></div>
          {/if}
          <div class="setting-row"><div><strong>Onboarding</strong><small>Replay the welcome flow, including CLI, account, and workspace checks.</small></div><button class="quiet" onclick={onReplayOnboarding}>Run onboarding again</button></div>
        {/if}
      </article>
    </div>
    {#if showCloudConsent}<div class="consent-sheet" role="dialog" aria-label="Cloud transcription consent"><p class="eyebrow">Per-session consent</p><h2>Cloud transcription fallback</h2><dl><div><dt>Provider</dt><dd>Managed OpenAI transcription</dd></div><div><dt>Audio destination</dt><dd>Provider processing endpoint</dd></div><div><dt>Retention</dt><dd>Zero data retention where supported</dd></div></dl><p>Raw audio must be sent to the named provider for transcription. The returned transcript is sanitized before any later cloud planning. Local Whisper remains the default path.</p><div><button class="deny" onclick={() => (showCloudConsent = false)}>Cancel</button><button class="primary" onclick={consentCloudFallback}>Consent for next session</button></div></div>{/if}
  {/if}
</section>
