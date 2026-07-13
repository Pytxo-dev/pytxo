<script lang="ts">
  import DeckToolbar from "../shell/DeckToolbar.svelte";
  import CommandBar from "../shell/CommandBar.svelte";
  import UpdateBanner from "../shell/UpdateBanner.svelte";
  import ActivitySidebar from "../panels/ActivitySidebar.svelte";
  import LogPanel from "../panels/LogPanel.svelte";
  import InspectorPanel from "../panels/InspectorPanel.svelte";
  import ProjectPathPanel from "../panels/ProjectPathPanel.svelte";
  import TopologyScene3D from "../topology/TopologyScene3D.svelte";
  import type { WorkspaceTab } from "../../lib/deck-store.svelte";
  import type { HitlDto, ProjectRootDto } from "../../lib/types";
  import type { DeckTheme } from "../../lib/theme";
  import { isLightDeck } from "../../lib/theme";
  import { Button } from "$lib/components/ui/button";

  let {
    deckTheme = $bindable("void" as DeckTheme),
    cmd = $bindable(""),
    tabs = $bindable([] as WorkspaceTab[]),
    activeTabId = $bindable(null as string | null),
    termEl = $bindable(undefined as HTMLDivElement | undefined),
    selectedTopologyNode = $bindable(null as string | null),
    logExpanded = $bindable(false),
    showWorkspaceSettings = $bindable(false),
    running = false,
    hitl,
    topologyLoading = false,
    fleetRuns = [],
    diffText = "",
    dryRunOut = "",
    signalRetries = [],
    tier = "core",
    maxAgents = 3,
    signedIn = false,
    cliMissing = false,
    cloudRunBadge = null,
    walletMicrocredits = null as number | null,
    permissionCeiling = null as string | null,
    subscriptionPortalUrl = null as string | null,
    networkIsolationBadge = null as string | null,
    onSelectRun,
    onSelectAgent,
    onDryRun,
    onDispatch,
    onStop,
    onRefresh,
    onAuthChange,
    onLoadDiff,
    onCommit,
    onRespondHitl,
    onRootsChange,
    onGoHome,
  }: {
    deckTheme?: DeckTheme;
    cmd?: string;
    tabs?: WorkspaceTab[];
    activeTabId?: string | null;
    termEl?: HTMLDivElement | undefined;
    selectedTopologyNode?: string | null;
    logExpanded?: boolean;
    showWorkspaceSettings?: boolean;
    running?: boolean;
    hitl: HitlDto[];
    topologyLoading?: boolean;
    fleetRuns?: import("../../lib/types").FleetRunDto[];
    diffText?: string;
    dryRunOut?: string;
    signalRetries?: string[];
    tier?: string;
    maxAgents?: number;
    signedIn?: boolean;
    cliMissing?: boolean;
    cloudRunBadge?: "cloud" | "fallback" | null;
    walletMicrocredits?: number | null;
    permissionCeiling?: string | null;
    subscriptionPortalUrl?: string | null;
    networkIsolationBadge?: string | null;
    onSelectRun: (tabId: string, runId: string) => void;
    onSelectAgent: (tabId: string, agentId: string) => void;
    onDryRun: () => void;
    onDispatch: () => void;
    onStop: () => void;
    onRefresh: () => void;
    onAuthChange: () => void;
    onLoadDiff: () => void;
    onCommit: () => void;
    onRespondHitl: (id: string, approve: boolean) => void;
    onRootsChange: (roots: ProjectRootDto[]) => void;
    onGoHome: () => void;
  } = $props();

  const activeTab = $derived(tabs.find((t) => t.id === activeTabId) ?? null);
  const runs = $derived(activeTab?.runs ?? []);
  const agents = $derived(activeTab?.agents ?? []);
  const structural = $derived(activeTab?.structural ?? null);
  const selectedRunId = $derived(activeTab?.selectedRunId ?? null);
  const selectedAgentId = $derived(activeTab?.selectedAgentId ?? null);
  const dispatchRepo = $derived(activeTab?.dispatchRepo ?? "");
  const projectId = $derived(activeTab?.projectId ?? null);
  const roots = $derived(activeTab?.roots ?? []);

  const selectedRun = $derived(runs.find((r) => r.id === selectedRunId) ?? null);
  const lightTheme = $derived(isLightDeck(deckTheme));

  function updateDispatchRepo(value: string) {
    if (!activeTab) return;
    tabs = tabs.map((t) => (t.id === activeTab.id ? { ...t, dispatchRepo: value } : t));
  }
</script>

<div class="deck-workspace">
  <DeckToolbar
    bind:theme={deckTheme}
    {tier}
    {maxAgents}
    {signedIn}
    {cliMissing}
    {cloudRunBadge}
    {walletMicrocredits}
    {permissionCeiling}
    {subscriptionPortalUrl}
    onRefresh={onRefresh}
    onAuthChange={onAuthChange}
  />
  <UpdateBanner />

  <div class="deck-workspace__main">
    <ActivitySidebar
      {runs}
      {agents}
      {fleetRuns}
      selectedRunId={selectedRunId}
      selectedAgentId={selectedAgentId}
      onSelectRun={(id) => activeTab && onSelectRun(activeTab.id, id)}
      onSelectAgent={(id) => activeTab && onSelectAgent(activeTab.id, id)}
      {networkIsolationBadge}
    />

    <div class="deck-workspace__stage">
      <div class="stage-toolbar">
        <Button
          variant="ghost"
          size="sm"
          onclick={() => (showWorkspaceSettings = !showWorkspaceSettings)}
        >
          {showWorkspaceSettings ? "Hide folders" : "Workspace folders"}
        </Button>
        <Button variant="ghost" size="sm" onclick={() => (logExpanded = !logExpanded)}>
          {logExpanded ? "Hide telemetry" : "Show telemetry"}
        </Button>
      </div>
      {#if showWorkspaceSettings}
        <div class="workspace-settings">
          {#if projectId}
            <ProjectPathPanel {projectId} {roots} onRootsChange={onRootsChange} />
          {:else}
            <div class="workspace-settings__simple">
              <h2 class="panel-title">Workspace folders</h2>
              <p class="workspace-settings__hint">
                This workspace is a single folder. Open a multi-root project from Workspaces home
                to add API, web, and shared paths under one run.
              </p>
              <code class="workspace-settings__path">{activeTab?.domainId ?? ""}</code>
              <Button variant="outline" size="sm" onclick={onGoHome}>Open Workspaces</Button>
            </div>
          {/if}
        </div>
      {/if}
      <TopologyScene3D
        {structural}
        loading={topologyLoading}
        bind:selectedNodeId={selectedTopologyNode}
        {deckTheme}
      />
      {#if logExpanded}
        <LogPanel bind:termEl {signalRetries} {lightTheme} />
      {/if}
    </div>

    <InspectorPanel
      {diffText}
      {dryRunOut}
      {hitl}
      selectedNodeId={selectedTopologyNode}
      {structural}
      isolationBackend={selectedRun?.isolation_backend || selectedRun?.isolation_mode || ""}
      onLoad={onLoadDiff}
      onCommit={onCommit}
      onRespondHitl={onRespondHitl}
    />
  </div>

  <CommandBar
    cmd={cmd}
    dispatchRepo={dispatchRepo}
    {running}
    onDispatchRepoChange={updateDispatchRepo}
    {onDryRun}
    {onDispatch}
    {onStop}
  />
</div>

<style>
  .deck-workspace {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }
  .deck-workspace__main {
    display: flex;
    flex: 1;
    min-height: 0;
  }
  .deck-workspace__stage {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
    background: var(--background);
  }
  .stage-toolbar {
    display: flex;
    justify-content: flex-end;
    gap: 0.25rem;
    padding: 0.5rem 0.75rem;
    border-bottom: 1px solid var(--border);
  }
  .workspace-settings {
    max-height: 220px;
    overflow: auto;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid var(--border);
    background: color-mix(in oklab, var(--card) 70%, transparent);
  }
  .workspace-settings__simple {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 28rem;
  }
  .workspace-settings__hint {
    margin: 0;
    font-size: 0.82rem;
    color: var(--muted-foreground);
    line-height: 1.45;
  }
  .workspace-settings__path {
    font-size: 0.72rem;
    word-break: break-all;
    padding: 0.4rem 0.5rem;
    border-radius: var(--panel-radius);
    background: color-mix(in oklab, var(--foreground) 5%, transparent);
  }
  .deck-workspace__stage :global(.log-pane) {
    flex: 0 0 200px;
    min-height: 0;
    margin: 0;
    border-top: 1px solid var(--border);
  }
</style>
