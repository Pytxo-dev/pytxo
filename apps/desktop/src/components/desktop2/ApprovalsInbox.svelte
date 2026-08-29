<script lang="ts">
  import { IconCheck, IconChevronRight, IconShieldLock, IconX } from "@tabler/icons-svelte";
  import { approvalPresentation } from "../../lib/approval-presentation";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { HitlDto } from "../../lib/types";

  let {
    open = false,
    snapshot,
    backend,
    onClose,
    onReviewRun,
    onApprovalsChanged,
  }: {
    open?: boolean;
    snapshot: DesktopSnapshot;
    backend: DesktopBackend;
    onClose: () => void;
    onReviewRun: (runId: string) => void;
    onApprovalsChanged: () => void | Promise<void>;
  } = $props();

  let dialog: HTMLDialogElement | undefined = $state();
  let selectedApprovalId = $state<string | null>(null);
  let deciding = $state(false);
  let decisionMessage = $state<string | null>(null);
  let decisionMessageTone = $state<"success" | "error" | null>(null);

  const openApprovals = $derived(snapshot.approvals);
  const selected: HitlDto | null = $derived(
    openApprovals.find((a) => a.id === selectedApprovalId) ?? openApprovals[0] ?? null,
  );
  const presentation = $derived(selected ? approvalPresentation(selected) : null);
  const selectedIndex = $derived(selected ? openApprovals.findIndex((a) => a.id === selected.id) : -1);
  const selectedDomain = $derived(
    selected?.domain_id ? snapshot.domains.find((d) => d.domain_id === selected.domain_id) ?? null : null,
  );
  const selectedRun = $derived(
    selectedDomain ? snapshot.runs.find((run) => run.repo_root === selectedDomain.repo_root) ?? null : null,
  );

  /**
   * The overlay is opened from the title bar, the boundary panel, and the `A`
   * chord, so the dialog element follows the `open` prop rather than owning the
   * decision itself.
   */
  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  });

  $effect(() => {
    if (openApprovals.length && !openApprovals.some((a) => a.id === selectedApprovalId)) {
      selectedApprovalId = openApprovals[0]?.id ?? null;
    }
  });

  function workspaceLabel(): string {
    return selectedDomain?.repo_root.split(/[\\/]/).pop() ?? selected?.domain_id ?? "Unknown";
  }

  function requestedAt(approval: HitlDto): string {
    const timestamp = Number(approval.created_at_ms);
    return Number.isFinite(timestamp) ? new Date(timestamp).toLocaleString() : "Unknown";
  }

  function isEditableTarget(target: EventTarget | null): boolean {
    if (!(target instanceof HTMLElement)) return false;
    return target.isContentEditable || ["INPUT", "TEXTAREA", "SELECT"].includes(target.tagName);
  }

  function selectRelative(offset: number) {
    if (!openApprovals.length || deciding) return;
    const current = selectedIndex >= 0 ? selectedIndex : 0;
    const next = (current + offset + openApprovals.length) % openApprovals.length;
    selectedApprovalId = openApprovals[next]?.id ?? null;
  }

  function onKeydown(event: KeyboardEvent) {
    if (!open || !selected || event.defaultPrevented || event.repeat || isEditableTarget(event.target)) return;
    const modifier = event.ctrlKey || event.metaKey;
    const key = event.key.toLowerCase();
    if (!modifier && !event.altKey && !event.shiftKey && (key === "j" || key === "k")) {
      event.preventDefault();
      selectRelative(key === "j" ? 1 : -1);
      return;
    }
    if (modifier && !event.altKey && !event.shiftKey && event.key === "Enter") {
      event.preventDefault();
      void resolve(true);
      return;
    }
    if (modifier && !event.altKey && !event.shiftKey && event.key === "Backspace") {
      event.preventDefault();
      void resolve(false);
    }
  }

  async function resolve(approve: boolean) {
    if (!selected || deciding) return;
    const resolved = selected;
    const shown = approvalPresentation(resolved);
    deciding = true;
    decisionMessage = null;
    decisionMessageTone = null;
    try {
      if (approve) await backend.approve(resolved.id, resolved.domain_id ?? null);
      else await backend.deny(resolved.id, resolved.domain_id ?? null);
      decisionMessage = `${approve ? "Approved" : "Denied"} ${shown.title}. ${approve ? shown.approvedMessage : shown.deniedMessage}`;
      decisionMessageTone = "success";
      selectedApprovalId = openApprovals.find((a) => a.id !== resolved.id)?.id ?? null;
      await onApprovalsChanged();
    } catch (error) {
      decisionMessage = `Could not ${approve ? "approve" : "deny"} ${shown.title}. ${error instanceof Error ? error.message : String(error)}`;
      decisionMessageTone = "error";
    } finally {
      deciding = false;
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<dialog bind:this={dialog} class="inbox-dialog" aria-label="Approvals inbox" onclose={onClose}>
  <header>
    <div><p>Waiting on you</p><h2>Approvals</h2></div>
    <span class="count">{openApprovals.length} open</span>
    <button class="close" onclick={onClose} aria-label="Close approvals"><IconX size={16} /></button>
  </header>

  {#if decisionMessage}
    <p
      class="decision-message"
      class:error={decisionMessageTone === "error"}
      role={decisionMessageTone === "error" ? "alert" : "status"}
    >{decisionMessage}</p>
  {/if}

  <div class="inbox-layout">
    <div class="queue">
      {#if openApprovals.length}
        {#each openApprovals as approval (approval.id)}
          {@const shown = approvalPresentation(approval)}
          <button
            class="queue-row"
            class:active={selected?.id === approval.id}
            aria-pressed={selected?.id === approval.id}
            onclick={() => (selectedApprovalId = approval.id)}
          >
            <span class="risk">{shown.category}</span>
            <strong>{shown.title}</strong>
            <small>{approval.agent_key} · {requestedAt(approval)}</small>
            <IconChevronRight size={15} />
          </button>
        {/each}
      {:else}
        <div class="empty"><strong>Inbox clear</strong><span>Requests appear here when an agent waits on you.</span></div>
      {/if}
    </div>

    <div class="detail">
      {#if selected && presentation}
        <div class="detail-title">
          <div><span>{presentation.category}</span><h3>{presentation.title}</h3></div>
          <span class="position" aria-live="polite">{selectedIndex + 1} of {openApprovals.length}</span>
        </div>
        <p class="reason">{selected.reason}</p>
        <dl>
          <div><dt>Requested by</dt><dd>{selected.agent_key}</dd></div>
          <div><dt>Workspace</dt><dd>{workspaceLabel()}</dd></div>
          <div><dt>Requested</dt><dd>{requestedAt(selected)}</dd></div>
          <div><dt>Action</dt><dd>{selected.action}</dd></div>
        </dl>
        <div class="consequence">
          <span><IconShieldLock size={16} /> What each choice does</span>
          <small>{presentation.consequence}</small>
        </div>
        <div class="evidence">
          <div>
            <strong>Run evidence</strong>
            {#if selectedRun}
              <small>{selectedRun.id} · {selectedRun.status} · {selectedRun.permission_profile ?? "profile not reported"}</small>
              <small>This request carries no run ID. Showing the latest run for {workspaceLabel()}.</small>
            {:else}
              <small>No run available for this workspace. Decide from the action and reason above.</small>
            {/if}
          </div>
          {#if selectedRun}
            <button class="quiet" onclick={() => { onReviewRun(selectedRun.id); onClose(); }}>
              Review run <IconChevronRight size={14} />
            </button>
          {/if}
        </div>
        <div class="actions">
          <button class="deny" disabled={deciding} onclick={() => resolve(false)}>
            {presentation.denyLabel}<kbd aria-hidden="true">Ctrl/⌘ ⌫</kbd>
          </button>
          <button class="primary" disabled={deciding} onclick={() => resolve(true)}>
            <IconCheck size={16} /> {presentation.approveLabel}<kbd aria-hidden="true">Ctrl/⌘ ↵</kbd>
          </button>
        </div>
      {:else}
        <div class="empty"><strong>Inbox clear</strong><span>All decisions have been resolved.</span></div>
      {/if}
    </div>
  </div>
</dialog>

<style>
  .inbox-dialog{position:fixed;inset:0;width:min(940px,92vw);max-width:none;max-height:82dvh;margin:auto;padding:0;border:1px solid var(--pytxo-line);border-radius:8px;background:var(--pytxo-surface-panel);color:var(--pytxo-text-strong);box-shadow:0 24px 80px color-mix(in oklab,var(--pytxo-surface-shell) 82%,transparent);overflow:hidden}
  .inbox-dialog::backdrop{background:color-mix(in oklab,black 62%,transparent)}
  header{display:flex;align-items:center;gap:12px;padding:14px 16px;border-bottom:1px solid var(--pytxo-line-soft)}
  header p{margin:0 0 4px;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.07em}
  header h2{margin:0;font-size:16px;font-weight:640;letter-spacing:-.02em}
  .count{margin-left:auto;padding:3px 8px;border:1px solid var(--state-attention);border-radius:3px;color:var(--state-attention);font:11px "IBM Plex Mono",monospace}
  .close{display:grid;width:28px;height:28px;place-items:center;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:transparent;color:var(--pytxo-text-muted);cursor:pointer}
  .close:hover{color:var(--pytxo-text-strong)}

  .decision-message{margin:0;padding:9px 16px;border-bottom:1px solid var(--pytxo-line-soft);color:var(--state-verified);font-size:12px}
  .decision-message.error{color:var(--state-refuted)}

  .inbox-layout{display:grid;grid-template-columns:minmax(230px,300px) minmax(0,1fr);min-height:0;max-height:calc(82dvh - 62px)}
  .queue{display:flex;flex-direction:column;overflow:auto;border-right:1px solid var(--pytxo-line-soft)}
  .queue-row{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:2px 8px;padding:10px 14px;border:0;border-bottom:1px solid var(--pytxo-line-soft);border-left:2px solid transparent;background:transparent;color:inherit;cursor:pointer;font-family:inherit;text-align:left}
  .queue-row:hover{background:color-mix(in oklab,var(--pytxo-surface-raised) 55%,transparent)}
  .queue-row.active{border-left-color:var(--state-attention);background:var(--pytxo-surface-active)}
  .queue-row:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .queue-row .risk{grid-column:1/-1;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.05em}
  .queue-row strong{overflow:hidden;font-size:12px;text-overflow:ellipsis;white-space:nowrap}
  .queue-row small{grid-column:1/-1;overflow:hidden;color:var(--pytxo-text-muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}
  .queue-row>:global(svg){align-self:center;color:var(--pytxo-text-muted)}

  .detail{display:flex;flex-direction:column;gap:13px;overflow:auto;padding:16px}
  .detail-title{display:flex;align-items:flex-start;justify-content:space-between;gap:12px}
  .detail-title span{color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.05em}
  .detail-title h3{margin:5px 0 0;font-size:16px;font-weight:620;letter-spacing:-.02em}
  .position{flex:none;color:var(--pytxo-text-muted);font-size:11px}
  .reason{margin:0;color:var(--pytxo-text-soft);font-size:12px;line-height:1.55}
  dl{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:1px;margin:0;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:var(--pytxo-line-soft);overflow:hidden}
  dl>div{padding:8px 10px;background:var(--pytxo-surface-input)}
  dt{color:var(--pytxo-text-muted);font-size:11px}
  dd{overflow:hidden;margin:3px 0 0;font:11px "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap}

  .consequence{display:flex;flex-direction:column;gap:5px;padding:11px;border:1px solid var(--pytxo-line-soft);border-left:2px solid var(--state-attention);border-radius:4px}
  .consequence span{display:flex;align-items:center;gap:7px;font-size:12px;font-weight:600}
  .consequence span>:global(svg){color:var(--state-attention)}
  .consequence small,.evidence small{color:var(--pytxo-text-muted);font-size:11px;line-height:1.45}

  .evidence{display:flex;align-items:flex-start;justify-content:space-between;gap:12px;padding:11px;border:1px solid var(--pytxo-line-soft);border-radius:4px}
  .evidence div{display:flex;min-width:0;flex-direction:column;gap:3px}
  .evidence strong{font-size:12px}

  .actions{display:flex;gap:9px;margin-top:auto;padding-top:4px}
  .actions button{display:flex;height:38px;flex:1;align-items:center;justify-content:center;gap:8px;border-radius:4px;font-size:12px;font-weight:620;cursor:pointer}
  .actions .deny{border:1px solid var(--pytxo-line);background:transparent;color:var(--pytxo-text-soft)}
  .actions .deny:hover:not(:disabled){border-color:var(--state-refuted);color:var(--state-refuted)}
  .actions .primary{border:1px solid transparent;background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell)}
  .actions button:disabled{cursor:not-allowed;opacity:.4}
  kbd{padding:1px 5px;border:1px solid currentColor;border-radius:3px;font:11px "IBM Plex Mono",monospace;opacity:.6}
  .quiet{height:28px;padding:0 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-soft);font-size:11px;cursor:pointer}

  .empty{display:flex;min-height:180px;flex-direction:column;align-items:center;justify-content:center;gap:6px;padding:20px;color:var(--pytxo-text-muted);text-align:center}
  .empty strong{color:var(--pytxo-text-soft);font-size:13px}
  .empty span{max-width:280px;font-size:12px;line-height:1.5}
  @media(max-width:820px){.inbox-layout{grid-template-columns:minmax(0,1fr);grid-template-rows:auto minmax(0,1fr)}.queue{max-height:180px;border-right:0;border-bottom:1px solid var(--pytxo-line-soft)}}
</style>
