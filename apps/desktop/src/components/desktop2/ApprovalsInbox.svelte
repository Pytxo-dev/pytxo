<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import IconCheck from "@tabler/icons-svelte/icons/check";
  import IconChevronRight from "@tabler/icons-svelte/icons/chevron-right";
  import IconShieldLock from "@tabler/icons-svelte/icons/shield-lock";
  import IconX from "@tabler/icons-svelte/icons/x";
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
    selected?.run_id
      ? snapshot.runs.find((run) => run.domain_id === selected.domain_id && run.id === selected.run_id) ?? null
      : null,
  );

  /**
   * The overlay is opened from the title bar, the boundary panel, and the `A`
   * chord, so the dialog element follows the `open` prop rather than owning the
   * decision itself.
   */
  $effect(() => {
    if (!dialog) return;
    if (open && !dialog.open) void withPreviewsHidden(() => { if (open && dialog?.isConnected && !dialog.open) dialog.showModal(); });
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
    // A queue response carries no reviewed candidate digest. Neither the
    // button nor its keyboard shortcut may authorize a repository flush.
    if (approve && shown.requiresCandidateReview) return;
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

<dialog bind:this={dialog} class="inbox-dialog" class:clear={!openApprovals.length} aria-label="Approvals inbox" onclose={onClose}>
  <header>
    <h2>Approvals</h2>
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

  <div class="inbox-layout" class:clear={!openApprovals.length}>
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
            <small>{approval.agent_id ?? approval.agent_key} · {requestedAt(approval)}</small>
            <IconChevronRight size={15} />
          </button>
        {/each}
      {/if}
    </div>

    <div class="detail">
      {#if selected && presentation}
        <div class="detail-body">
        <div class="detail-title">
          <div><span>{presentation.category}</span><h3>{presentation.title}</h3></div>
          <span class="position" aria-live="polite">{selectedIndex + 1} of {openApprovals.length}</span>
        </div>
        <p class="reason">{selected.reason}</p>
        <dl>
          <div><dt>Requested by</dt><dd>{selected.agent_id ?? selected.agent_key}</dd></div>
          <div><dt>Workspace</dt><dd>{workspaceLabel()}</dd></div>
          <div><dt>Run</dt><dd>{selected.run_id ?? "Not recorded"}</dd></div>
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
              <small>Exact run and agent: {selected.run_id} · {selected.agent_id ?? selected.agent_key}</small>
            {:else if selected.run_id}
              <small>Run {selected.run_id} is not present in this partial snapshot. No substitute run is shown.</small>
            {:else}
              <small>This legacy request has no causal run ID. Decide from the action and reason above; no substitute run is shown.</small>
            {/if}
          </div>
          {#if selectedRun}
            <button class="quiet" onclick={() => { onReviewRun(selectedRun.id); onClose(); }}>
              Review run <IconChevronRight size={14} />
            </button>
          {/if}
        </div>
        </div>
        <div class="actions">
          <button class="deny" disabled={deciding} onclick={() => resolve(false)}>
            {presentation.denyLabel}<kbd aria-hidden="true">Ctrl/⌘ ⌫</kbd>
          </button>
          {#if presentation.requiresCandidateReview}
            <button class="primary" disabled={deciding || !selectedRun} onclick={() => {
              if (selectedRun) { onReviewRun(selectedRun.id); onClose(); }
            }}>
              <IconChevronRight size={16} /> {presentation.approveLabel}
            </button>
          {:else}
          <button class="primary" disabled={deciding} onclick={() => resolve(true)}>
            <IconCheck size={16} /> {presentation.approveLabel}<kbd aria-hidden="true">Ctrl/⌘ ↵</kbd>
          </button>
          {/if}
        </div>
      {:else}
        <div class="empty"><strong>Inbox clear</strong><span>No agents are waiting for a decision. Requests will appear here.</span></div>
      {/if}
    </div>
  </div>
</dialog>

<style>
  .inbox-dialog{position:fixed;inset:0 0 0 auto;width:min(560px,100vw);max-width:100vw;margin:0;padding:0;border:0;border-left:1px solid var(--pytxo-line);border-radius:0;background:var(--pytxo-surface-panel);color:var(--pytxo-text-strong);box-shadow:-12px 0 40px color-mix(in oklab,var(--pytxo-surface-shell) 35%,transparent);overflow:hidden}
  .inbox-dialog[open]{display:flex;flex-direction:column;height:100dvh;max-height:100dvh}
  .inbox-dialog[open].clear{height:min(300px,calc(100dvh - 48px));width:min(560px,92vw);inset:0;margin:auto;border:1px solid var(--pytxo-line);border-radius:8px}
  .inbox-layout.clear{grid-template-columns:minmax(0,1fr);grid-template-rows:minmax(0,1fr)}
  .inbox-layout.clear .queue{display:none}
  .clear .detail .empty{flex:1;min-height:0}
  .inbox-dialog::backdrop{background:color-mix(in oklab,black 24%,transparent)}
  header{flex:none;display:flex;align-items:center;gap:12px;padding:14px 16px;border-bottom:1px solid var(--pytxo-line-soft)}
  header h2{margin:0;font-size:16px;font-weight:640;letter-spacing:-.02em}
  .count{margin-left:auto;padding:3px 8px;border:1px solid var(--state-attention);border-radius:3px;color:var(--state-attention);font:11px "IBM Plex Mono",monospace}
  .close{display:grid;width:40px;height:40px;place-items:center;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:transparent;color:var(--pytxo-text-muted);cursor:pointer}
  .close:hover{color:var(--pytxo-text-strong)}

  .decision-message{flex:none;max-height:90px;overflow:auto;margin:0;padding:9px 16px;border-bottom:1px solid var(--pytxo-line-soft);color:var(--state-verified);font-size:12px}
  .decision-message.error{color:var(--state-refuted)}

  .inbox-layout{display:grid;grid-template-columns:minmax(0,1fr);grid-template-rows:auto minmax(0,1fr);min-height:0;flex:1;overflow:hidden}
  .queue{display:flex;flex-direction:column;max-height:132px;overflow:auto;border-bottom:1px solid var(--pytxo-line-soft)}
  .queue-row{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:2px 8px;padding:10px 14px;border:0;border-bottom:1px solid var(--pytxo-line-soft);border-left:2px solid transparent;background:transparent;color:inherit;cursor:pointer;font-family:inherit;text-align:left}
  .queue-row:hover{background:color-mix(in oklab,var(--pytxo-surface-raised) 55%,transparent)}
  .queue-row.active{border-left-color:var(--state-attention);background:var(--pytxo-surface-active)}
  .queue-row:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .queue-row .risk{grid-column:2;grid-row:1;align-self:center;padding:0;background:transparent;color:var(--pytxo-text-soft);font-size:11px}
  .queue-row strong{grid-column:1;grid-row:1;overflow:hidden;font-size:13px;text-overflow:ellipsis;white-space:nowrap}
  .queue-row small{grid-column:1;grid-row:2;overflow:hidden;color:var(--pytxo-text-muted);font-size:11px;text-overflow:ellipsis;white-space:nowrap}
  .queue-row>:global(svg){grid-column:2;grid-row:2;justify-self:end;align-self:center;color:var(--pytxo-text-muted)}

  .detail{display:flex;flex-direction:column;min-height:0;min-width:0;overflow:hidden}
  .detail-body{display:flex;flex-direction:column;gap:16px;overflow:auto;overscroll-behavior:contain;min-height:0;padding:20px;scrollbar-gutter:stable}
  .detail-body > * { flex-shrink: 0; }
  .detail-title{display:flex;align-items:flex-start;justify-content:space-between;gap:12px}
  .detail-title span{color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.05em}
  .detail-title h3{margin:5px 0 0;font-size:16px;font-weight:620;letter-spacing:-.02em}
  .position{flex:none;color:var(--pytxo-text-muted);font-size:11px}
  .reason{margin:0;color:var(--pytxo-text-soft);font-size:13px;line-height:1.6;overflow-wrap:anywhere}
  dl{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:1px;margin:0;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:var(--pytxo-line-soft);overflow:hidden}
  dl>div{padding:8px 10px;background:var(--pytxo-surface-input)}
  dt{color:var(--pytxo-text-muted);font-size:11px}
  dd{margin:4px 0 0;font:12px "IBM Plex Mono",monospace;overflow-wrap:anywhere}

  .consequence{display:flex;flex-direction:column;gap:8px;padding:12px 0;border-block:1px solid var(--pytxo-line-soft)}
  .consequence span{display:flex;align-items:center;gap:7px;font-size:12px;font-weight:600}
  .consequence span>:global(svg){color:var(--state-attention)}
  .consequence small,.evidence small{color:var(--pytxo-text-soft);font-size:12px;line-height:1.5}

  .evidence{display:flex;align-items:flex-start;justify-content:space-between;gap:12px;padding:0}
  .evidence div{display:flex;min-width:0;flex-direction:column;gap:3px}
  .evidence strong{font-size:12px}

  .actions{display:flex;flex:none;flex-wrap:wrap;gap:8px;margin-top:auto;padding:12px 20px;border-top:1px solid var(--pytxo-line-soft);background:var(--pytxo-surface-panel)}
  .actions button{display:flex;min-height:40px;padding:8px 10px;flex:1;align-items:center;justify-content:center;gap:8px;border-radius:4px;font-size:12px;font-weight:620;cursor:pointer}
  .actions .deny{border:1px solid var(--pytxo-line);background:transparent;color:var(--pytxo-text-soft)}
  .actions .deny:hover:not(:disabled){border-color:var(--state-refuted);color:var(--state-refuted)}
  .actions .primary{border:1px solid transparent;background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell)}
  .actions button:disabled{cursor:not-allowed;opacity:.4}
  kbd{padding:1px 5px;border:1px solid currentColor;border-radius:3px;font:11px "IBM Plex Mono",monospace}
  .quiet{display:inline-flex;align-items:center;gap:6px;flex:none;min-height:40px;padding:0 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-soft);font-size:11px;cursor:pointer}

  .empty{display:flex;min-height:180px;flex-direction:column;align-items:center;justify-content:center;gap:6px;padding:20px;color:var(--pytxo-text-muted);text-align:center}
  .empty strong{color:var(--pytxo-text-soft);font-size:13px}
  .empty span{max-width:280px;font-size:12px;line-height:1.5}
  @media(max-width:900px){.actions kbd{display:none}}
  @media(max-width:680px){.queue{max-height:110px}.detail-body{padding:16px}.actions{padding:12px 16px}.evidence{flex-wrap:wrap}}
</style>
