<script lang="ts">
  import { IconBrandOpenai, IconCheck, IconChevronRight, IconCloud, IconFolder, IconPlugConnected, IconPlus, IconSearch, IconSettings, IconShieldLock } from "@tabler/icons-svelte";
  import type { DesktopSnapshot } from "../../lib/desktop-backend";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import type { AppRoute } from "../../lib/navigation.svelte";
  import { applyDeckTheme } from "../../lib/theme";
  let { route, snapshot, backend, onRoute }: { route: AppRoute; snapshot: DesktopSnapshot; backend: DesktopBackend; onRoute: (route: AppRoute) => void } = $props();
  let query = $state("");
  let resolved = $state<string[]>([]);
  let selectedSetting = $state("Appearance");
  let voiceModelPath = $state<string | null>(null);
  let voiceInstalling = $state(false);
  let showCloudConsent = $state(false);
  let openedWorkspace = $state<string | null>(null);

  async function resolveApproval(approve: boolean) {
    const approval = snapshot.approvals.find((item) => !resolved.includes(item.id));
    if (!approval) return;
    if (approve) await backend.approve(approval.id, approval.domain_id ?? null);
    else await backend.deny(approval.id, approval.domain_id ?? null);
    resolved = [...resolved, approval.id];
  }

  async function installVoiceModel() {
    voiceInstalling = true;
    try { voiceModelPath = await backend.installVoiceModel(); } finally { voiceInstalling = false; }
  }

  async function openWorkspace() {
    openedWorkspace = await backend.openWorkspace();
  }
</script>

<section class="screen collection-screen">
  <header class="screen-heading">
    <div><p class="eyebrow">Pytxo Desktop</p><h1>{route === "run-review" ? "Run Review" : route[0].toUpperCase() + route.slice(1)}</h1><p>{route === "workspaces" ? "Execution domains, roots, and health in one catalog." : route === "runs" ? "Every orchestration run, from dispatch through recovery." : route === "approvals" ? "Human decisions with the context needed to act confidently." : route === "integrations" ? "ADEs, providers, MCP connections, and cloud sandboxes." : "Tune appearance, privacy, execution, permissions, and Voice."}</p></div>
    {#if route === "workspaces"}<button class="primary" onclick={openWorkspace}><IconPlus size={16} /> Add workspace</button>{/if}
  </header>

  {#if route === "workspaces"}
    <div class="toolbar"><label><IconSearch size={16} /><input bind:value={query} placeholder="Search workspaces" aria-label="Search workspaces" /></label><button class="quiet">Health: all</button><button class="quiet">Recently active</button></div>
    {#if openedWorkspace}<p class="workspace-opened">Added {openedWorkspace}</p>{/if}<div class="catalog-grid">{#each snapshot.domains.filter((domain) => domain.repo_root.toLowerCase().includes(query.toLowerCase())) as domain}<article class="workspace-card"><div class="workspace-icon"><IconFolder size={19} /></div><div class="workspace-title"><h2>{domain.repo_root.split(/[\\/]/).pop()}</h2><span class="healthy"><i></i>{domain.status}</span></div><p>{domain.repo_root}</p><div class="workspace-stats"><span><small>Active runs</small><strong>{domain.active_runs}</strong></span><span><small>Approvals</small><strong>{domain.hitl_pending}</strong></span><span><small>Policy</small><strong>Orbit</strong></span></div><button class="card-link" onclick={() => onRoute("operations")}>Open workspace <IconChevronRight size={16} /></button></article>{/each}</div>
  {:else if route === "runs"}
    <article class="panel table-panel"><div class="panel-head"><h2>Run history</h2><div><button class="quiet" onclick={() => onRoute("topology-focus")}>Topology Focus</button> <button class="quiet" onclick={() => onRoute("run-review")}>Run Review</button></div></div><div class="data-table" role="table"><div class="table-row table-header" role="row"><span>Run</span><span>Workspace</span><span>Status</span><span>Policy</span><span>Estimate</span></div>{#each snapshot.runs as run}<button class="table-row" role="row" onclick={() => onRoute("run-review")}><span class="mono">{run.id}</span><span>{run.repo_root.split(/[\\/]/).pop()}</span><span><i class:active={run.status === "running"}></i>{run.status}</span><span>{run.permission_profile}</span><span>${(run.estimated_cost_usd ?? 0).toFixed(2)} <IconChevronRight size={15} /></span></button>{/each}</div></article>
  {:else if route === "approvals"}
    <div class="approval-layout"><article class="panel inbox"><div class="panel-head"><h2>Inbox</h2><span>{snapshot.approvals.length - resolved.length} open</span></div>{#each snapshot.approvals.filter((item) => !resolved.includes(item.id)) as approval}<button class="inbox-row"><span class="risk">Review</span><strong>{approval.action}</strong><small>{approval.agent_key} · {new Date(Number(approval.created_at_ms)).toLocaleString()}</small><IconChevronRight size={16} /></button>{/each}</article><article class="panel decision-detail"><p class="eyebrow">Decision context</p><h2>{snapshot.approvals.find((item) => !resolved.includes(item.id))?.action ?? "Inbox clear"}</h2><p>{snapshot.approvals.find((item) => !resolved.includes(item.id))?.reason ?? "All decisions have been resolved."}</p><div class="diff-summary"><span><IconShieldLock size={17} /> Blast Shield</span><strong>Review required</strong><small>Pytxo will enforce the execution-domain permission profile when this workspace is flushed. Open Run Review for verified file-level evidence.</small></div><div class="decision-actions"><button class="deny" disabled={resolved.length === snapshot.approvals.length} onclick={() => resolveApproval(false)}>Deny</button><button class="primary" disabled={resolved.length === snapshot.approvals.length} onclick={() => resolveApproval(true)}><IconCheck size={16} /> Approve & flush</button></div></article></div>
  {:else if route === "integrations"}
    <div class="integration-sections"><article class="panel"><div class="panel-head"><div><p class="eyebrow">Agent development environments</p><h2>ADE registry</h2></div><button class="quiet"><IconPlus size={15} /> Add custom</button></div><div class="integration-grid">{#each [{name:"Cursor CLI", meta:"cursor agent", status:"Ready"},{name:"Codex CLI",meta:"codex",status:"Ready"},{name:"Claude Code",meta:"claude",status:"Detected"},{name:"OpenCode",meta:"opencode",status:"Not installed"}] as item}<div class="integration-card"><div class="provider-icon"><IconBrandOpenai size={19} /></div><p><strong>{item.name}</strong><small class="mono">{item.meta}</small></p><span class:healthy={item.status !== "Not installed"}>{item.status}</span><IconChevronRight size={16} /></div>{/each}</div></article><article class="panel integration-row"><div><IconPlugConnected size={20} /><p><strong>MCP connections</strong><small>3 servers connected</small></p></div><div><IconCloud size={20} /><p><strong>Cloud sandboxes</strong><small>Local fallback ready</small></p></div></article></div>
  {:else}
    <div class="settings-layout">
      <nav aria-label="Settings sections">
        {#each ["Appearance","Voice","Privacy","Execution defaults","Permissions","Notifications","Storage","Account & billing"] as item}
          <button class:active={item === selectedSetting} onclick={() => (selectedSetting = item)}><IconSettings size={16} />{item}</button>
        {/each}
      </nav>
      <article class="panel settings-panel">
        <p class="eyebrow">{selectedSetting === "Voice" ? "Local-first capture" : selectedSetting === "Privacy" ? "Sovereign Shield" : "Desktop preferences"}</p>
        <h2>{selectedSetting}</h2>
        {#if selectedSetting === "Appearance"}
          <div class="setting-row"><div><strong>Theme</strong><small>Obsidian is canonical; light mode is token-complete.</small></div><div class="segmented"><button onclick={() => applyDeckTheme("void")}>Dark</button><button onclick={() => applyDeckTheme("light")}>Light</button><button onclick={() => applyDeckTheme(matchMedia("(prefers-color-scheme: light)").matches ? "light" : "void")}>System</button></div></div>
          <div class="setting-row"><div><strong>Density</strong><small>Comfortable information density for long sessions.</small></div><select aria-label="Density"><option>Comfortable</option><option>Compact</option></select></div>
          <div class="setting-row"><div><strong>Reduced motion</strong><small>Respect the operating system preference.</small></div><button class="toggle" aria-label="Reduced motion"><i></i></button></div>
        {:else if selectedSetting === "Voice"}
          <div class="setting-row"><div><strong>Local transcription model</strong><small>{voiceModelPath ?? "base.en · 148 MB · not installed"}</small></div><button class="quiet" disabled={voiceInstalling} onclick={installVoiceModel}>{voiceInstalling ? "Installing…" : "Install base.en"}</button></div>
          <div class="setting-row"><div><strong>Capture behavior</strong><small>Click-to-record and press-and-hold; audio is never written to disk.</small></div><select aria-label="Voice capture behavior"><option>Both</option><option>Press and hold</option><option>Click to record</option></select></div>
          <div class="setting-row"><div><strong>Cloud fallback</strong><small>Requires explicit consent for every recording session.</small></div><button class="quiet" onclick={() => (showCloudConsent = true)}>Review consent</button></div>
        {:else if selectedSetting === "Privacy"}
          <div class="setting-row"><div><strong>Raw audio retention</strong><small>Bounded memory only; cleared after completion, failure, or cancellation.</small></div><strong>Never stored</strong></div>
          <div class="setting-row"><div><strong>Transcript sanitization</strong><small>Sovereign Shield runs before cloud planning.</small></div><strong>Enabled</strong></div>
        {:else}
          <div class="setting-row"><div><strong>{selectedSetting}</strong><small>Uses organization policy and local execution-domain defaults.</small></div><button class="quiet">Configure</button></div>
        {/if}
      </article>
    </div>
    {#if showCloudConsent}<div class="consent-sheet" role="dialog" aria-label="Cloud transcription consent"><p class="eyebrow">Per-session consent</p><h2>Cloud transcription fallback</h2><dl><div><dt>Provider</dt><dd>Managed OpenAI transcription</dd></div><div><dt>Audio destination</dt><dd>Provider processing endpoint</dd></div><div><dt>Retention</dt><dd>Zero data retention where supported</dd></div></dl><p>Raw audio must be sent to the named provider for transcription. The returned transcript is sanitized before any later cloud planning.</p><div><button class="deny" onclick={() => (showCloudConsent = false)}>Cancel</button><button class="primary" onclick={() => (showCloudConsent = false)}>Consent for next session</button></div></div>{/if}
  {/if}
</section>
