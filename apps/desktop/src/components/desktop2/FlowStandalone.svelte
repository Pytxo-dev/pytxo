<script lang="ts">
  import { onMount } from "svelte";
  import { createDesktopBackend } from "../../lib/desktop-backend";
  import TitleBar from "../shell/TitleBar.svelte";
  import FlowScreen from "./FlowScreen.svelte";
  import "./desktop2-shared.css";

  const backend = createDesktopBackend();
  let domains = $state<{ domain_id: string; repo_root: string }[]>([]);
  let preferredDomainId = $state<string | null>(null);

  async function refreshDomains() {
    const snap = await backend.loadSnapshot({ includeAgents: false });
    domains = snap.domains.map((d) => ({ domain_id: d.domain_id, repo_root: d.repo_root }));
  }

  async function addWorkspace() {
    try {
      const opened = await backend.openWorkspace();
      if (opened) preferredDomainId = opened;
      await refreshDomains();
    } catch {
      /* dialog cancelled or preview */
    }
  }

  onMount(() => {
    void refreshDomains();
  });
</script>

<div class="flow-standalone">
  <TitleBar title="Pytxo Flow" />
  <div class="flow-standalone__body desktop2">
    <FlowScreen
      {backend}
      {domains}
      preferredDomainId={preferredDomainId}
      onAddWorkspace={addWorkspace}
    />
  </div>
</div>

<style>
  .flow-standalone {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--background);
    color: var(--foreground);
  }
  .flow-standalone__body {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }
</style>
