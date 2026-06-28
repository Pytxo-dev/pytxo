<script lang="ts">
  import Header from "../shell/Header.svelte";
  import CommandBar from "../shell/CommandBar.svelte";
  import UpdateBanner from "../shell/UpdateBanner.svelte";
  import NavPanel from "../panels/NavPanel.svelte";
  import LogPanel from "../panels/LogPanel.svelte";
  import InspectorPanel from "../panels/InspectorPanel.svelte";
  import TopologyScene3D from "../topology/TopologyScene3D.svelte";
  import type { AgentDto, DomainDto, HitlDto, RunDto, StructuralGraphDto } from "../../lib/types";
  import type { DeckTheme } from "../../lib/theme";
  import { isLightDeck } from "../../lib/theme";

  let {
    deckTheme = $bindable("void" as DeckTheme),
    cmd = $bindable(""),
    dispatchRepo = $bindable(""),
    termEl = $bindable(undefined as HTMLDivElement | undefined),
    selectedDomainId = $bindable(null as string | null),
    selectedRunId = $bindable(null as string | null),
    selectedAgentId = $bindable(null as string | null),
    selectedTopologyNode = $bindable(null as string | null),
    logExpanded = $bindable(false),
    domains,
    runs,
    agents,
    hitl,
    structural,
    topologyLoading = false,
    fleetRuns = [],
    workspaceLabel = "",
    workspacePath = "",
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
    selectedRun = null as RunDto | null,
    onSelectRun,
    onSelectAgent,
    onChangeWorkspace,
    onDryRun,
    onDispatch,
    onStop,
    onRefresh,
    onAuthChange,
    onLoadDiff,
    onCommit,
    onRespondHitl,
  }: {
    deckTheme?: DeckTheme;
    cmd?: string;
    dispatchRepo?: string;
    termEl?: HTMLDivElement | undefined;
    selectedDomainId?: string | null;
    selectedRunId?: string | null;
    selectedAgentId?: string | null;
    selectedTopologyNode?: string | null;
    logExpanded?: boolean;
    domains: DomainDto[];
    runs: RunDto[];
    agents: AgentDto[];
    hitl: HitlDto[];
    structural: StructuralGraphDto | null;
    topologyLoading?: boolean;
    fleetRuns?: import("../../lib/types").FleetRunDto[];
    workspaceLabel?: string;
    workspacePath?: string;
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
    selectedRun?: RunDto | null;
    onSelectRun: (id: string) => void;
    onSelectAgent: (id: string) => void;
    onChangeWorkspace: (path: string) => void;
    onDryRun: () => void;
    onDispatch: () => void;
    onStop: () => void;
    onRefresh: () => void;
    onAuthChange: () => void;
    onLoadDiff: () => void;
    onCommit: () => void;
    onRespondHitl: (id: string, approve: boolean) => void;
  } = $props();

  const lightTheme = $derived(isLightDeck(deckTheme));
</script>

<div class="dashboard">
  <Header
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

  <div class="dashboard__main">
    <NavPanel
      {workspaceLabel}
      {workspacePath}
      {domains}
      {runs}
      {agents}
      {fleetRuns}
      bind:selectedDomainId
      bind:selectedRunId
      bind:selectedAgentId
      onSelectRun={onSelectRun}
      onSelectAgent={onSelectAgent}
      onChangeWorkspace={onChangeWorkspace}
      {networkIsolationBadge}
    />

    <div class="dashboard__center">
      <TopologyScene3D
        {structural}
        loading={topologyLoading}
        bind:selectedNodeId={selectedTopologyNode}
        {deckTheme}
      />
      <div class="telemetry-bar">
        <button type="button" onclick={() => (logExpanded = !logExpanded)}>
          {logExpanded ? "Hide telemetry" : "Show telemetry"}
        </button>
      </div>
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

  <CommandBar bind:cmd bind:dispatchRepo {onDryRun} {onDispatch} {onStop} />
</div>

<style>
  .dashboard {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-height: 0;
  }
  .dashboard__main {
    display: flex;
    flex: 1;
    min-height: 0;
    gap: 0;
  }
  .dashboard__center {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 0;
  }
  .telemetry-bar {
    display: flex;
    justify-content: flex-end;
    padding: 0.25rem 0.75rem;
  }
  .telemetry-bar button {
    min-height: 36px;
    border-radius: 8px;
  }
  .dashboard__center :global(.log-pane) {
    flex: 0 0 200px;
    min-height: 0;
    margin: 0 0.5rem 0.5rem;
    border-radius: 12px;
  }
</style>
