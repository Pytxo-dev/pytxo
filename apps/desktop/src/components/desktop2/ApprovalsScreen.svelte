<script lang="ts">
  import { onMount } from "svelte";
  import { IconCheck, IconChevronRight, IconShieldLock } from "@tabler/icons-svelte";
  import { approvalPresentation } from "../../lib/approval-presentation";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { HitlDto } from "../../lib/types";

  let {
    snapshot,
    backend,
    onReviewRun,
    onApprovalsChanged,
  }: {
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    onReviewRun: (runId: string) => void;
    onApprovalsChanged: () => void | Promise<void>;
  } = $props();

  let selectedApprovalId = $state<string | null>(null);
  let deciding = $state(false);
  let decisionMessage = $state<string | null>(null);
  let decisionMessageTone = $state<"success" | "error" | null>(null);

  const openApprovals = $derived(snapshot.approvals);
  const selectedApproval: HitlDto | null = $derived(
    openApprovals.find((a) => a.id === selectedApprovalId) ?? openApprovals[0] ?? null,
  );
  const selectedPresentation = $derived(
    selectedApproval ? approvalPresentation(selectedApproval) : null,
  );
  const selectedIndex = $derived(
    selectedApproval ? openApprovals.findIndex((approval) => approval.id === selectedApproval.id) : -1,
  );
  const selectedDomain = $derived(
    selectedApproval?.domain_id
      ? snapshot.domains.find((domain) => domain.domain_id === selectedApproval.domain_id) ?? null
      : null,
  );
  const selectedRun = $derived(
    selectedDomain
      ? snapshot.runs.find((run) => run.repo_root === selectedDomain.repo_root) ?? null
      : null,
  );

  function workspaceLabel(): string {
    return selectedDomain?.repo_root.split(/[\\/]/).pop() ?? selectedApproval?.domain_id ?? "Unknown";
  }

  function requestedAt(approval: HitlDto): string {
    const timestamp = Number(approval.created_at_ms);
    return Number.isFinite(timestamp) ? new Date(timestamp).toLocaleString() : "Unknown";
  }

  function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
  }

  function selectRelativeApproval(offset: number) {
    if (!openApprovals.length || deciding) return;
    const current = selectedIndex >= 0 ? selectedIndex : 0;
    const next = (current + offset + openApprovals.length) % openApprovals.length;
    selectedApprovalId = openApprovals[next]?.id ?? null;
  }

  function onApprovalKeydown(event: KeyboardEvent) {
    if (!selectedApproval || event.defaultPrevented || event.repeat || isEditableTarget(event.target)) return;
    const modifier = event.ctrlKey || event.metaKey;
    if (!modifier && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "j") {
      event.preventDefault();
      selectRelativeApproval(1);
      return;
    }
    if (!modifier && !event.altKey && !event.shiftKey && event.key.toLowerCase() === "k") {
      event.preventDefault();
      selectRelativeApproval(-1);
      return;
    }
    if (modifier && !event.altKey && !event.shiftKey && event.key === "Enter") {
      event.preventDefault();
      void resolveApproval(true);
      return;
    }
    if (modifier && !event.altKey && !event.shiftKey && event.key === "Backspace") {
      event.preventDefault();
      void resolveApproval(false);
    }
  }

  async function resolveApproval(approve: boolean) {
    if (!selectedApproval || deciding) return;
    const resolved = selectedApproval;
    const presentation = approvalPresentation(resolved);
    deciding = true;
    decisionMessage = null;
    decisionMessageTone = null;
    try {
      if (approve) await backend.approve(resolved.id, resolved.domain_id ?? null);
      else await backend.deny(resolved.id, resolved.domain_id ?? null);
      decisionMessage = `${approve ? "Approved" : "Denied"} ${presentation.title}. ${approve ? presentation.approvedMessage : presentation.deniedMessage}`;
      decisionMessageTone = "success";
      selectedApprovalId = openApprovals.find((approval) => approval.id !== resolved.id)?.id ?? null;
      await onApprovalsChanged();
    } catch (error) {
      decisionMessage = `Could not ${approve ? "approve" : "deny"} ${presentation.title}. ${error instanceof Error ? error.message : String(error)}`;
      decisionMessageTone = "error";
    } finally {
      deciding = false;
    }
  }

  $effect(() => {
    if (openApprovals.length && !openApprovals.some((a) => a.id === selectedApprovalId)) {
      selectedApprovalId = openApprovals[0]?.id ?? null;
    }
  });

  onMount(() => {
    window.addEventListener("keydown", onApprovalKeydown);
    return () => window.removeEventListener("keydown", onApprovalKeydown);
  });
</script>

<section class="screen collection-screen">
  <header class="screen-heading">
    <div>
      <h1>Approvals</h1>
    </div>
  </header>

  {#if decisionMessage}
    <p
      class="voice-state-message decision-message"
      class:error={decisionMessageTone === "error"}
      role={decisionMessageTone === "error" ? "alert" : "status"}
    >{decisionMessage}</p>
  {/if}
  <div class="approval-layout">
    <article class="panel inbox">
      <div class="panel-head"><h2>Inbox</h2><span>{openApprovals.length} open</span></div>
      {#if openApprovals.length}
        {#each openApprovals as approval}
          {@const presentation = approvalPresentation(approval)}
          <button
            class="inbox-row"
            class:active={selectedApproval?.id === approval.id}
            aria-pressed={selectedApproval?.id === approval.id}
            onclick={() => (selectedApprovalId = approval.id)}
          >
            <span class="risk">{presentation.category}</span>
            <strong>{presentation.title}</strong>
            <small>{approval.agent_key} · {requestedAt(approval)}</small>
            <IconChevronRight size={16} />
          </button>
        {/each}
      {:else}
        <div class="empty"><strong>Inbox clear</strong><span>New approval requests appear here when agents wait on you.</span></div>
      {/if}
    </article>
    <article class="panel decision-detail">
      {#if selectedApproval && selectedPresentation}
        <div class="decision-title">
          <div>
            <span>{selectedPresentation.category}</span>
            <h2>{selectedPresentation.title}</h2>
          </div>
          <span class="selection-position" aria-live="polite">{selectedIndex + 1} of {openApprovals.length}</span>
        </div>
        <p class="decision-reason">{selectedApproval.reason}</p>
        <dl class="decision-meta">
          <div><dt>Requested by</dt><dd class="mono">{selectedApproval.agent_key}</dd></div>
          <div><dt>Workspace</dt><dd>{workspaceLabel()}</dd></div>
          <div><dt>Requested</dt><dd>{requestedAt(selectedApproval)}</dd></div>
          <div><dt>Action</dt><dd class="mono">{selectedApproval.action}</dd></div>
        </dl>
        <div class="diff-summary">
          <span><IconShieldLock size={17} /> Permission gate</span>
          <strong>{selectedPresentation.approveLabel} · {selectedPresentation.denyLabel}</strong>
          <small>{selectedPresentation.consequence}</small>
        </div>
        <div class="decision-evidence">
          <div>
            <strong>Latest run evidence</strong>
            {#if selectedRun}
              <small><span class="mono">{selectedRun.id}</span> · {selectedRun.status} · {selectedRun.permission_profile ?? "unknown"} profile</small>
              <small>This request does not include a run ID. Showing the latest run for {workspaceLabel()}.</small>
            {:else}
              <small>No run is available for this workspace. Decide from the exact action and reason above.</small>
            {/if}
          </div>
          {#if selectedRun}
            <button class="quiet" onclick={() => onReviewRun(selectedRun.id)}>Review latest run <IconChevronRight size={14} /></button>
          {/if}
        </div>
        <div class="decision-actions">
          <button class="deny" disabled={deciding} onclick={() => resolveApproval(false)}>{selectedPresentation.denyLabel}<kbd aria-hidden="true">Ctrl/⌘ ⌫</kbd></button>
          <button class="primary" disabled={deciding} onclick={() => resolveApproval(true)}><IconCheck size={16} /> {selectedPresentation.approveLabel}<kbd aria-hidden="true">Ctrl/⌘ ↵</kbd></button>
        </div>
      {:else}
        <h2>Inbox clear</h2>
        <p>All decisions have been resolved.</p>
      {/if}
    </article>
  </div>
</section>
