<script lang="ts">
  import { IconBell, IconHistory } from "@tabler/icons-svelte";
  import { ROUTE_LABELS, type AppRoute } from "../../lib/navigation.svelte";

  let {
    route,
    approvalsCount,
    hypervisorOnline,
    onOpenHistory,
    onOpenNotifications,
  }: {
    route: AppRoute;
    approvalsCount: number;
    hypervisorOnline: boolean;
    onOpenHistory: () => void;
    onOpenNotifications: () => void;
  } = $props();
</script>

<div class="app-bar">
  <div class="breadcrumbs">
    <span>Pytxo</span><i>/</i><strong>{ROUTE_LABELS[route]}</strong>
  </div>
  <div class="app-actions">
    <span class="connection" class:offline={!hypervisorOnline}>
      <i></i> {hypervisorOnline ? "Local hypervisor" : "Hypervisor unreachable"}
    </span>
    <button aria-label="Run history" title="Run history" onclick={onOpenHistory}>
      <IconHistory size={17} />
    </button>
    <button aria-label="Notifications" title="Approvals" onclick={onOpenNotifications}>
      <IconBell size={17} />
      {#if approvalsCount}<i class="badge"></i>{/if}
    </button>
  </div>
</div>

<style>
  .app-bar {
    height: 47px;
    flex: none;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid #1e2026;
    padding: 0 22px;
  }
  .breadcrumbs {
    display: flex;
    gap: 8px;
    align-items: center;
    font-size: 11px;
    color: #5e6571;
  }
  .breadcrumbs i {
    font-style: normal;
    color: #343842;
  }
  .breadcrumbs strong {
    color: #b4bac3;
    font-weight: 550;
  }
  .app-actions {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .connection {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-right: 8px;
    color: #717885;
    font-size: 10px;
  }
  .connection i {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #3ed7a1;
    box-shadow: 0 0 0 3px rgba(62, 215, 161, 0.08);
  }
  .connection.offline {
    color: #d98994;
  }
  .connection.offline i {
    background: #df6576;
    box-shadow: 0 0 0 3px rgba(223, 101, 118, 0.1);
  }
  .app-actions button {
    position: relative;
    width: 30px;
    height: 30px;
    border-radius: 5px;
    border: 0;
    background: none;
    color: #707783;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: background-color 150ms ease, color 150ms ease;
  }
  .app-actions button:hover {
    background: #14161c;
    color: #ccd1d8;
  }
  .app-actions button:focus-visible {
    outline: 2px solid #38d6c1;
    outline-offset: 1px;
  }
  .app-actions button .badge {
    position: absolute;
    right: 5px;
    top: 5px;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: #eeac47;
  }

  :global(html[data-chroma-theme="light"]) .app-bar {
    border-color: var(--pytxo-line);
  }
</style>
