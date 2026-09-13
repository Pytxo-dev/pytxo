<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import { onMount, tick } from "svelte";
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconArrowLeft from "@tabler/icons-svelte/icons/arrow-left";
  import IconCheck from "@tabler/icons-svelte/icons/check";
  import IconFileMinus from "@tabler/icons-svelte/icons/file-minus";
  import IconFilePlus from "@tabler/icons-svelte/icons/file-plus";
  import IconFileText from "@tabler/icons-svelte/icons/file-text";
  import IconFingerprint from "@tabler/icons-svelte/icons/fingerprint";
  import IconLoader2 from "@tabler/icons-svelte/icons/loader-2";
  import IconRefresh from "@tabler/icons-svelte/icons/refresh";
  import IconShieldCheck from "@tabler/icons-svelte/icons/shield-check";
  import IconTrash from "@tabler/icons-svelte/icons/trash";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import { recordedWorkerLabel, reviewPresentation } from "../../lib/review-state";
  import type {
    AgentDto,
    EnforcementSurface,
    PreparedContentChunkDto,
    PreparedRunFile,
    RunDto,
    RunReviewDto,
  } from "../../lib/types";

  let {
    backend,
    run,
    domainId,
    onBack,
    onChanged,
  }: {
    backend: DesktopBackend;
    run: RunDto;
    domainId: string | null;
    onBack: () => void;
    onChanged: () => void | Promise<void>;
  } = $props();

  let review = $state<RunReviewDto | null>(null);
  let agents = $state<AgentDto[]>([]);
  let loading = $state(true);
  let actionPending = $state(false);
  let error = $state("");
  let notice = $state("");
  let confirmApply = $state(false);
  let confirmDiscard = $state(false);
  let backButton = $state<HTMLButtonElement | null>(null);
  let primaryTrigger = $state<HTMLButtonElement | null>(null);
  let cancelApplyButton = $state<HTMLButtonElement | null>(null);
  let applyExactPackageButton = $state<HTMLButtonElement | null>(null);
  let discardTrigger = $state<HTMLButtonElement | null>(null);
  let keepReviewButton = $state<HTMLButtonElement | null>(null);
  let discardPermanentlyButton = $state<HTMLButtonElement | null>(null);
  type ReviewSide = "before" | "after";
  type PreparedContentState = {
    segments: Uint8Array[];
    loadedByteCount: number;
    byteCount: number;
    binary: boolean;
    displayByteCount: number;
    complete: boolean;
    loading: boolean;
    error: string;
  };
  let preparedContent = $state<Record<string, PreparedContentState>>({});
  let selectedPreparedPath = $state<string | null>(null);
  let contentSelection = 0;
  type QueuedContentRead = {
    selection: number;
    start: () => Promise<PreparedContentChunkDto>;
    resolve: (chunk: PreparedContentChunkDto | null) => void;
    reject: (cause: unknown) => void;
  };
  let activeContentReads = 0;
  let queuedContentReads: QueuedContentRead[] = [];

  const manifest = $derived(review?.prepared_manifest ?? null);
  const candidateEvidence = $derived(manifest?.candidate_verification);
  const candidatePassed = $derived(manifest?.version === 3 && candidateEvidence?.version === 1 && candidateEvidence.checks.length > 0 && candidateEvidence.checks.every((check) => check.passed));
  const presentation = $derived(
    reviewPresentation({
      apply_status: review?.apply_status ?? run.apply_status ?? "unavailable",
      recovery_state: review?.recovery_state ?? null,
      last_apply_error: review?.last_apply_error ?? null,
      candidate_verified: candidatePassed,
      prepared_file_count: manifest?.files.length ?? null,
    }),
  );
  const tasks = $derived(review?.plan.waves.flat() ?? []);
  const receiptRows = $derived.by((): Array<[string, EnforcementSurface]> => {
    const receipt = review?.enforcement.run;
    if (!receipt) return [];
    return [
      ["Workspace", receipt.workspace_isolation],
      ["Host filesystem", receipt.host_filesystem_boundary],
      ["Network", receipt.network],
      ["Apply", receipt.apply_boundary],
    ];
  });
  const allAgentsSucceeded = $derived(
    agents.length > 0 && agents.every((agent) => agent.exit_code === 0),
  );
  const terminalWithoutAction = $derived(
    presentation.primaryAction === null &&
      (presentation.state === "applied" ||
        presentation.state === "discarded" ||
        presentation.state === "non_flushable" ||
        presentation.state === "not_applicable" ||
        presentation.state === "unsupported"),
  );
  const applyDisabledReason = $derived(
    terminalWithoutAction
      ? ""
      : actionPending
      ? "Another review action is in progress."
      : !presentation.applyAllowed
        ? presentation.state === "recovery_required"
          ? "Apply is blocked until recovery is reconciled."
          : presentation.detail
        : !allAgentsSucceeded
          ? "Every agent must exit successfully before Apply."
          : "",
  );

  async function loadReview({ focus = false }: { focus?: boolean } = {}) {
    loading = true;
    error = "";
    try {
      [review, agents] = await Promise.all([
        backend.runReview(run.id, domainId),
        backend.listAgents(run.id, domainId),
      ]);
      const firstFile = review?.prepared_manifest?.files[0];
      if (firstFile) {
        await selectPreparedFile(firstFile, true);
      } else {
        contentSelection += 1;
        cancelQueuedContentReads();
        selectedPreparedPath = null;
        preparedContent = {};
      }
      if (focus) {
        await tick();
        backButton?.focus();
      }
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      loading = false;
    }
  }

  async function runPrimaryAction() {
    if (!review || actionPending) return;
    actionPending = true;
    error = "";
    notice = "";
    try {
      if (presentation.primaryAction === "refresh") {
        await backend.refreshRunReview(run.id, domainId);
        notice = "Immutable review refreshed against the current checkout.";
      } else if (
        presentation.primaryAction === "apply" ||
        presentation.primaryAction === "retry"
      ) {
        if (!presentation.applyAllowed || !allAgentsSucceeded) return;
        await backend.applyRunChanges(run.id, domainId);
      } else if (presentation.primaryAction === "reconcile") {
        const outcome = await backend.reconcileRunRecovery(run.id, domainId);
        notice =
          outcome.outcome === "recovery_required"
            ? "Recovery still requires manual attention."
            : "Recovery reconciled. Review state reloaded.";
      }
      await onChanged();
      await loadReview();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
      try {
        await onChanged();
      } catch {
        /* Preserve the original action error; the immediate domain event is a second refresh path. */
      }
      await loadReview();
    } finally {
      actionPending = false;
    }
  }

  async function requestPrimaryAction() {
    if (
      presentation.primaryAction === "apply" ||
      presentation.primaryAction === "retry"
    ) {
      if (!presentation.applyAllowed || !allAgentsSucceeded || actionPending) return;
      await withPreviewsHidden(() => { confirmApply = true; });
      await tick();
      cancelApplyButton?.focus();
      return;
    }
    await runPrimaryAction();
  }

  async function applyExactPackage() {
    confirmApply = false;
    await runPrimaryAction();
  }

  async function closeApplyDialog() {
    confirmApply = false;
    await tick();
    primaryTrigger?.focus();
  }

  function handleApplyDialogKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      void closeApplyDialog();
      return;
    }
    if (event.key !== "Tab" || !cancelApplyButton || !applyExactPackageButton) return;
    if (event.shiftKey && document.activeElement === cancelApplyButton) {
      event.preventDefault();
      applyExactPackageButton.focus();
    } else if (!event.shiftKey && document.activeElement === applyExactPackageButton) {
      event.preventDefault();
      cancelApplyButton.focus();
    }
  }

  async function discardReview() {
    if (!presentation.discardAllowed || actionPending) return;
    actionPending = true;
    error = "";
    try {
      await backend.discardRunReview(run.id, domainId);
      confirmDiscard = false;
      notice = "Prepared review discarded.";
      await onChanged();
      await loadReview();
    } catch (cause) {
      error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      actionPending = false;
    }
  }

  async function openDiscardDialog() {
    await withPreviewsHidden(() => { confirmDiscard = true; });
    await tick();
    keepReviewButton?.focus();
  }

  async function closeDiscardDialog() {
    confirmDiscard = false;
    await tick();
    discardTrigger?.focus();
  }

  function handleDiscardDialogKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      void closeDiscardDialog();
      return;
    }
    if (event.key !== "Tab" || !keepReviewButton || !discardPermanentlyButton) return;
    if (event.shiftKey && document.activeElement === keepReviewButton) {
      event.preventDefault();
      discardPermanentlyButton.focus();
    } else if (!event.shiftKey && document.activeElement === discardPermanentlyButton) {
      event.preventDefault();
      keepReviewButton.focus();
    }
  }

  function fileLabel(file: PreparedRunFile) {
    return file.kind === "add" ? "Added" : file.kind === "delete" ? "Deleted" : "Modified";
  }

  function contentKey(file: PreparedRunFile, side: ReviewSide) {
    return `${file.path}\u0000${side}`;
  }

  function decodeBase64(value: string): Uint8Array {
    const binary = atob(value);
    return Uint8Array.from(binary, (character) => character.charCodeAt(0));
  }

  function flattenSegments(state: PreparedContentState, limit = state.loadedByteCount) {
    const flattened = new Uint8Array(Math.min(limit, state.loadedByteCount));
    let written = 0;
    for (const segment of state.segments) {
      if (written === flattened.length) break;
      const slice = segment.subarray(0, flattened.length - written);
      flattened.set(slice, written);
      written += slice.length;
    }
    return flattened;
  }

  function textContent(state: PreparedContentState) {
    return new TextDecoder("utf-8", { fatal: false }).decode(flattenSegments(state));
  }

  function hexContent(state: PreparedContentState) {
    return Array.from(flattenSegments(state, state.displayByteCount))
      .map((byte) => byte.toString(16).padStart(2, "0"))
      .join(" ");
  }

  function cancelQueuedContentReads() {
    const cancelled = queuedContentReads;
    queuedContentReads = [];
    for (const item of cancelled) item.resolve(null);
  }

  function pumpContentReads() {
    while (activeContentReads < 2 && queuedContentReads.length > 0) {
      const item = queuedContentReads.shift()!;
      if (item.selection !== contentSelection) {
        item.resolve(null);
        continue;
      }
      activeContentReads += 1;
      void item
        .start()
        .then(item.resolve, item.reject)
        .finally(() => {
          activeContentReads -= 1;
          pumpContentReads();
        });
    }
  }

  function scheduleContentRead(
    selection: number,
    start: () => Promise<PreparedContentChunkDto>,
  ): Promise<PreparedContentChunkDto | null> {
    if (selection !== contentSelection) return Promise.resolve(null);
    return new Promise((resolve, reject) => {
      queuedContentReads.push({ selection, start, resolve, reject });
      pumpContentReads();
    });
  }

  async function selectPreparedFile(file: PreparedRunFile, force = false) {
    if (!force && selectedPreparedPath === file.path) return;
    const selection = ++contentSelection;
    cancelQueuedContentReads();
    selectedPreparedPath = file.path;
    preparedContent = {};
    const sides: ReviewSide[] = [
      ...(file.before_sha256 ? (["before"] as ReviewSide[]) : []),
      ...(file.after_sha256 ? (["after"] as ReviewSide[]) : []),
    ];
    await Promise.all(
      sides.map((side) => loadPreparedContent(file, side, false, selection)),
    );
  }

  function showAllLoadedBinary(file: PreparedRunFile, side: ReviewSide) {
    const key = contentKey(file, side);
    const state = preparedContent[key];
    if (!state) return;
    preparedContent = {
      ...preparedContent,
      [key]: { ...state, displayByteCount: state.loadedByteCount },
    };
  }

  async function loadPreparedContent(
    file: PreparedRunFile,
    side: ReviewSide,
    expand: boolean,
    selection = contentSelection,
  ) {
    if (selection !== contentSelection || selectedPreparedPath !== file.path) return;
    const key = contentKey(file, side);
    const existing = preparedContent[key];
    if (existing?.loading || existing?.complete) return;
    let state: PreparedContentState = existing ?? {
      segments: [],
      loadedByteCount: 0,
      byteCount: side === "before" ? file.before_byte_count : file.after_byte_count,
      binary: (side === "before" ? file.before_is_binary : file.after_is_binary) ?? true,
      displayByteCount: 256,
      complete: false,
      loading: false,
      error: "",
    };
    const segments = [...state.segments];
    state = { ...state, segments };
    state = { ...state, loading: true, error: "" };
    preparedContent = { ...preparedContent, [key]: state };
    try {
      do {
        const chunk = await scheduleContentRead(
          selection,
          () => backend.runReviewContent(
            run.id,
            file.path,
            side,
            state.loadedByteCount,
            64 * 1024,
            domainId,
          ),
        );
        if (!chunk) return;
        const decoded = decodeBase64(chunk.data_base64);
        const expectedDigest = side === "before" ? file.before_sha256 : file.after_sha256;
        const expectedByteCount = side === "before" ? file.before_byte_count : file.after_byte_count;
        const expectedBinary = side === "before" ? file.before_is_binary : file.after_is_binary;
        const expectedPackageDigest = review?.prepared_digest ?? manifest?.package_digest;
        const identityMatches =
          chunk.run_id === run.id &&
          chunk.package_digest === expectedPackageDigest &&
          chunk.path === file.path &&
          chunk.side === side &&
          chunk.digest === expectedDigest &&
          chunk.byte_count === expectedByteCount &&
          chunk.binary === expectedBinary &&
          chunk.offset === state.loadedByteCount &&
          chunk.length === decoded.length &&
          (chunk.length > 0 || chunk.complete) &&
          chunk.next_offset === chunk.offset + chunk.length &&
          chunk.next_offset <= chunk.byte_count &&
          chunk.complete === (chunk.next_offset === chunk.byte_count);
        if (!identityMatches) {
          throw new Error("Exact review content identity changed; refresh the prepared review.");
        }
        if (selection !== contentSelection || selectedPreparedPath !== file.path) return;
        segments.push(decoded);
        state = {
          ...state,
          loadedByteCount: state.loadedByteCount + decoded.length,
          byteCount: chunk.byte_count,
          binary: chunk.binary,
          complete: chunk.complete,
        };
        if (!expand || state.complete) {
          preparedContent = { ...preparedContent, [key]: state };
        }
      } while (expand && !state.complete);
    } catch (cause) {
      if (selection !== contentSelection || selectedPreparedPath !== file.path) return;
      state = { ...state, error: cause instanceof Error ? cause.message : String(cause) };
    } finally {
      if (selection === contentSelection && selectedPreparedPath === file.path) {
        preparedContent = { ...preparedContent, [key]: { ...state, loading: false } };
      }
    }
  }

  function formatPreparedAt(value: string | null | undefined) {
    if (!value) return "Not recorded";
    return new Intl.DateTimeFormat("en", {
      month: "short",
      day: "numeric",
      year: "numeric",
      hour: "numeric",
      minute: "2-digit",
      timeZone: "UTC",
      timeZoneName: "short",
    }).format(new Date(value));
  }

  function attemptOutcomeLabel(outcome: string) {
    return outcome === "committed"
      ? "committed"
      : outcome === "rolled_back"
        ? "rolled back"
        : outcome === "recovery_required"
          ? "recovery required"
          : "interrupted";
  }

  onMount(() => {
    void loadReview({ focus: true });
  });
</script>

<section class="screen review-screen" aria-labelledby="run-review-title">
  <section class="review-decision" aria-label="Review decision">
  <header class="review-header">
    <div>
      <button class="back" bind:this={backButton} onclick={onBack}>
        <IconArrowLeft size={15} /> Back to runs
      </button>
      <h1 id="run-review-title">Run Review</h1>
      <p class="review-identity">{run.id} · {run.repo_root.split(/[\\/]/).pop()}</p>
    </div>
    <div class="review-actions">
      {#if presentation.primaryLabel}
        <button
          bind:this={primaryTrigger}
          class={presentation.primaryAction === "apply" || presentation.primaryAction === "retry"
            ? "primary primary-cue"
            : "primary"}
          disabled={actionPending ||
            (presentation.primaryAction !== "refresh" &&
              presentation.primaryAction !== "reconcile" &&
              (!presentation.applyAllowed || !allAgentsSucceeded))}
          title={applyDisabledReason || presentation.detail}
          aria-describedby={(presentation.primaryAction === "apply" ||
            presentation.primaryAction === "retry") && applyDisabledReason
            ? "apply-disabled-reason"
            : undefined}
          onclick={requestPrimaryAction}
        >
          {#if actionPending || presentation.busy}
            <IconLoader2 size={15} class="review-spinner" />
          {:else if presentation.primaryAction === "refresh"}
            <IconRefresh size={15} />
          {:else}
            <IconCheck size={15} />
          {/if}
          {presentation.primaryLabel}
        </button>
        {#if presentation.primaryAction === "refresh" || presentation.primaryAction === "reconcile"}
          <button
            class="apply-refusal"
            disabled
            title={applyDisabledReason}
            aria-describedby="apply-disabled-reason"
          >
            <IconCheck size={15} /> Apply reviewed changes
          </button>
        {/if}
      {:else if presentation.state === "applying"}
        <button
          class="primary"
          disabled
          title={presentation.detail}
          aria-describedby={applyDisabledReason ? "apply-disabled-reason" : undefined}
        >
          <IconLoader2 size={15} class="review-spinner" /> Applying…
        </button>
      {:else}
        <button
          class="primary"
          disabled
          title={presentation.detail}
          aria-describedby={applyDisabledReason ? "apply-disabled-reason" : undefined}
        >
          <IconCheck size={15} /> Apply reviewed changes
        </button>
      {/if}
      <button
        bind:this={discardTrigger}
        class="danger-quiet"
        disabled={!presentation.discardAllowed || actionPending}
        title={presentation.discardAllowed ? "Permanently remove the prepared package" : presentation.detail}
        onclick={openDiscardDialog}
      >
        <IconTrash size={15} /> Discard review
      </button>
    </div>
  </header>

  <div
    class={`review-status state-${presentation.state}`}
    aria-live="polite"
    aria-busy={presentation.busy}
  >
    {#if presentation.busy}
      <IconLoader2
        size={17}
        class="review-spinner"
        aria-label={presentation.state === "preparing"
          ? "Preparing immutable review"
          : "Applying reviewed changes"}
      />
    {:else if presentation.state === "verification_required" || presentation.state === "stale" || presentation.state === "review_failed" || presentation.state === "recovery_required"}
      <IconAlertTriangle size={17} />
    {:else}
      <IconShieldCheck size={17} />
    {/if}
    <div>
      <strong>{presentation.title}</strong>
      <span>{presentation.detail}</span>
    </div>
  </div>

  {#if review && manifest}
    <div class="decision-evidence">
      <p class="package-identity"><IconFingerprint size={14} /><span>Package</span><code>{review.prepared_digest ?? manifest.package_digest}</code></p>
      <p class="check-summary" class:verified={candidatePassed}>
        <strong>{candidatePassed ? "Combined checks: passed" : "Combined checks: not verified"}</strong>
        <span>{candidatePassed ? `${candidateEvidence?.checks.length} command(s) on this candidate` : "Task success alone does not verify the combined changes."}</span>
      </p>
    </div>
  {/if}

  {#if applyDisabledReason}
    <p class="action-explanation" id="apply-disabled-reason">{applyDisabledReason}</p>
  {/if}
  {#if notice}<p class="action-notice" aria-live="polite">{notice}</p>{/if}
  {#if error}<p class="action-error" role="alert">{error}</p>{/if}

  </section>

  {#if loading}
    <div class="review-loading" aria-label="Loading Run Review">
      <span></span><span></span><span></span>
    </div>
  {:else if review && manifest}
    <div class="review-grid">
      <article class="panel files-panel">
        <div class="panel-title files-title">
          <div><p class="eyebrow">Exact package contents</p><h2>Prepared changes</h2></div>
          <div class="summary">
            <span class="add">{manifest.summary.added} added</span>
            <span class="modify">{manifest.summary.modified} modified</span>
            <span class="delete">{manifest.summary.deleted} deleted</span>
          </div>
        </div>
        <div class="file-list">
          {#each manifest.files as file (file.path)}
            <div class={`file-row ${file.kind}`}>
              <header>
                {#if file.kind === "add"}<IconFilePlus size={16} />
                {:else if file.kind === "delete"}<IconFileMinus size={16} />
                {:else}<IconFileText size={16} />{/if}
                <span class="kind">{fileLabel(file)}</span>
                <div>
                  <strong>{file.path}</strong>
                  <small>{file.task_id} · {file.agent_id}</small>
                </div>
                <span class="bytes">{file.byte_count.toLocaleString()} B</span>
                <button
                  class="inspect-file"
                  class:selected={selectedPreparedPath === file.path}
                  aria-label={`Inspect exact content for ${file.path}`}
                  aria-expanded={selectedPreparedPath === file.path}
                  onclick={() => void selectPreparedFile(file)}
                >
                  {selectedPreparedPath === file.path ? "Selected" : "Inspect"}
                </button>
              </header>
              {#if selectedPreparedPath === file.path}
                <div class="exact-diff" class:single={file.kind !== "modify"}>
                  {#each (["before", "after"] as ReviewSide[]) as side}
                    {@const exists = side === "before" ? !!file.before_sha256 : !!file.after_sha256}
                    {@const content = preparedContent[contentKey(file, side)]}
                    {#if exists}
                      <section class={`diff-side ${side}`} aria-label={`${side} content for ${file.path}`}>
                        <div class="diff-label">
                          <strong>{side === "before" ? "Before" : "After"}</strong>
                          <span>{content?.byteCount.toLocaleString() ?? "0"} bytes · {content?.binary ? "binary" : "UTF-8 text"}</span>
                        </div>
                        {#if content?.error}
                          <p class="content-error" role="alert">{content.error}</p>
                        {:else if content?.binary}
                          <p class="binary-label">Exact binary bytes · hexadecimal preview</p>
                          <pre class="binary-content">{hexContent(content)}{content.displayByteCount < content.loadedByteCount || !content.complete ? " …" : ""}</pre>
                          {#if content.displayByteCount < content.loadedByteCount}
                            <button class="expand-content" onclick={() => showAllLoadedBinary(file, side)}>
                              Show all loaded binary bytes
                            </button>
                          {/if}
                        {:else if content}
                          <pre class="text-content">{textContent(content)}{content.complete ? "" : "\n… exact content continues"}</pre>
                        {:else}
                          <p class="content-loading">Loading exact prepared content…</p>
                        {/if}
                        {#if content && !content.complete}
                          <button class="expand-content" disabled={content.loading} onclick={() => loadPreparedContent(file, side, true)}>
                            {content.loading ? "Loading exact content…" : "Load full exact content"}
                          </button>
                        {/if}
                      </section>
                    {/if}
                  {/each}
                </div>
              {/if}
              <details class="digest-details">
                <summary>Digests and mode metadata</summary>
                <dl>
                  <div><dt>Before SHA-256</dt><dd>{file.before_sha256 ?? "—"}</dd></div>
                  <div><dt>After SHA-256</dt><dd>{file.after_sha256 ?? "—"}</dd></div>
                  <div><dt>Mode</dt><dd>{file.before_mode ?? "portable"} → {file.after_mode ?? "portable"}</dd></div>
                </dl>
              </details>
            </div>
          {/each}
        </div>
      </article>

      <aside class="review-support" aria-label="Review evidence and ownership">
      <article class="panel evidence-panel">
        <div class="panel-title"><p class="eyebrow">Immutable package</p><h2>Review evidence</h2></div>
        <dl class="evidence-list">
          <div><dt><IconShieldCheck size={14} /> Root</dt><dd>{run.repo_root}</dd></div>
          <div><dt>Profile</dt><dd>{review.enforcement.run.effective_profile}</dd></div>
          <div><dt>Paths</dt><dd>{manifest.summary.added + manifest.summary.modified + manifest.summary.deleted} · {manifest.summary.added} added · {manifest.summary.modified} modified · {manifest.summary.deleted} deleted</dd></div>
          <div><dt>State</dt><dd>{presentation.state}</dd></div>
          <div><dt>Combined candidate checks</dt><dd class="candidate-checks">
            {#if candidatePassed && candidateEvidence}
              Passed · {candidateEvidence.checks.length} command{candidateEvidence.checks.length === 1 ? "" : "s"} on this combined candidate. Verified {formatPreparedAt(candidateEvidence.verified_at)}.
              <ul>{#each candidateEvidence.checks as check}<li><code>{check.command}</code> · {check.task_id}</li>{/each}</ul>
              {#if candidateEvidence.exclusions.length}<span>Excluded from inventory: {candidateEvidence.exclusions.join(", ")}.</span>{/if}
            {:else if candidateEvidence}
              Not verified. The combined candidate receipt is incomplete, unsupported, or includes a failed check.
            {:else}
              Not verified. Task checks ran in separate workspaces; this package binds the reviewed bytes, not a passing combined check.
            {/if}
          </dd></div>
          <div><dt>Base revision</dt><dd>{manifest.base_revision}</dd></div>
          <div><dt>Prepared</dt><dd>{formatPreparedAt(review.prepared_at ?? manifest.prepared_at)}</dd></div>
          <div><dt>Execution domain</dt><dd>{review.enforcement.run.execution_domain}</dd></div>
        </dl>
        <div class="receipt-grid">
          {#each receiptRows as [label, receipt]}
            <div>
              <span class={`receipt-dot ${receipt.status}`}></span>
              <strong>{label}</strong>
              <small>{receipt.status} · {receipt.mechanism}</small>
            </div>
          {/each}
        </div>
        <details class="enforcement-details">
          <summary>How enforcement works</summary>
          {#each receiptRows as [label, receipt]}<p><strong>{label}</strong> · {receipt.detail}</p>{/each}
        </details>
      </article>

      <article class="panel plan-panel">
        <div class="panel-title"><p class="eyebrow">Plan / DAG</p><h2>Ownership path</h2></div>
        <div class="task-list">
          {#each tasks as task (task.task_id)}
            <div>
              <span>W{task.wave + 1}</span>
              <p><strong>{task.task_id}</strong><small>{recordedWorkerLabel(agents, run, task.task_id)} · {task.paths.join(", ")}</small></p>
              <em>{task.depends_on.length ? `after ${task.depends_on.join(", ")}` : "root task"}</em>
            </div>
          {/each}
        </div>
      </article>

      </aside>

      <article class="panel attempts-panel">
        <div class="panel-title"><p class="eyebrow">Apply audit</p><h2>Attempt history</h2></div>
        {#each review.apply_attempts as attempt (attempt.attempt_id)}
          <div
            class="attempt"
            class:success={attempt.outcome === "committed"}
            class:failure={attempt.outcome !== "committed"}
          >
            {#if attempt.outcome === "committed"}
              <IconCheck size={15} />
            {:else}
              <IconAlertTriangle size={15} />
            {/if}
            <p>
              <strong>{attemptOutcomeLabel(attempt.outcome)}</strong>
              <span>{attempt.error_message ?? `${attempt.phase} journal evidence`}</span>
              {#if attempt.error_code}
                <small class="audit-code">Audit code <code>{attempt.error_code}</code></small>
              {/if}
              <small>{attempt.created_at ?? "time unavailable"} · {attempt.attempt_id} · phase {attempt.phase} · rollback {attempt.rollback_confirmed ? "confirmed" : "not confirmed"}</small>
            </p>
          </div>
        {:else}
          <div class="empty compact"><strong>No Apply attempts yet</strong><span>The first attempt will be recorded here.</span></div>
        {/each}
      </article>
    </div>
  {:else}
    <article class="panel unavailable">
      <IconAlertTriangle size={20} />
      <div><h2>Prepared package unavailable</h2><p>{error || "This run has no immutable review manifest."}</p></div>
    </article>
  {/if}
</section>

{#if confirmApply}
  <div class="dialog-backdrop" role="presentation" onkeydown={handleApplyDialogKeydown}>
    <div class="confirm-dialog apply-dialog" role="dialog" aria-modal="true" aria-labelledby="apply-title">
      <IconShieldCheck size={22} />
      <div>
        <h2 id="apply-title">Apply exact reviewed package?</h2>
        <p>
          Pytxo will recheck and write {manifest?.files.length ?? 0} reviewed
          {manifest?.files.length === 1 ? " path" : " paths"} to the primary checkout.
          Unrelated paths are left alone.
        </p>
        {#if review?.prepared_digest}
          <code>{review.prepared_digest}</code>
        {/if}
      </div>
      <div class="dialog-actions">
        <button bind:this={cancelApplyButton} onclick={closeApplyDialog}>Cancel</button>
        <button
          bind:this={applyExactPackageButton}
          class="confirm-apply primary-cue"
          onclick={applyExactPackage}
          disabled={actionPending}
        >Apply exact package</button>
      </div>
    </div>
  </div>
{/if}

{#if confirmDiscard}
  <div class="dialog-backdrop" role="presentation" onkeydown={handleDiscardDialogKeydown}>
    <div class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="discard-title">
      <IconAlertTriangle size={22} />
      <div>
        <h2 id="discard-title">Discard prepared review?</h2>
        <p>This permanently removes the staged package and retained run workspaces. It does not change the primary checkout.</p>
      </div>
      <div class="dialog-actions">
        <button bind:this={keepReviewButton} onclick={closeDiscardDialog}>Keep review</button>
        <button
          bind:this={discardPermanentlyButton}
          class="danger"
          onclick={discardReview}
          disabled={actionPending}
        >Discard permanently</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .review-screen button,
  .confirm-dialog button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    min-height: var(--pytxo-control-height);
    padding: 0 12px;
    border: 1px solid var(--pytxo-line);
    border-radius: var(--pytxo-control-radius);
    background: var(--pytxo-surface-raised);
    color: var(--pytxo-text-strong);
    font-family: inherit;
    font-size: 12px;
    font-weight: 500;
    line-height: 1.4;
    cursor: pointer;
    transition: background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), border-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), color var(--pytxo-motion-fast) var(--pytxo-motion-ease);
  }
  .review-screen button:hover:not(:disabled),
  .confirm-dialog button:hover:not(:disabled) { background: var(--pytxo-surface-hover); border-color: var(--pytxo-text-muted); }
  .review-screen button:disabled,
  .confirm-dialog button:disabled { cursor: not-allowed; opacity: .45; }
  .review-screen .primary { background: var(--pytxo-text-strong); color: var(--pytxo-surface-shell); }
  .review-screen .primary:hover:not(:disabled) { background: var(--pytxo-text-body); }
  .review-screen { padding-bottom: 32px; container-type: inline-size; container-name: run-review; }
  .review-decision { padding: 0 0 12px; border-bottom: 1px solid var(--pytxo-line); }
  .decision-evidence { display: flex; justify-content: space-between; align-items: flex-start; flex-wrap: wrap; gap: 8px 20px; padding-top: 10px; }
  .package-identity { display: flex; align-items: center; flex-wrap: wrap; gap: 7px; min-width: 0; margin: 0; color: var(--pytxo-text-muted); font-size: 12px; }
  .package-identity code { color: var(--pytxo-text-body); overflow-wrap: anywhere; font: 12px "IBM Plex Mono", monospace; }
  .check-summary { display: grid; gap: 2px; margin: 0; font-size: 12px; color: var(--state-unknown); }
  .check-summary.verified { color: var(--state-verified); }
  .check-summary span { color: var(--pytxo-text-muted); }
  .enforcement-details { padding: 10px 12px; border-top: 1px solid var(--pytxo-line-soft); color: var(--pytxo-text-muted); font-size: 12px; }
  .enforcement-details summary { cursor: pointer; color: var(--pytxo-text-body); }
  .enforcement-details summary:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 3px; }
  .review-support .evidence-list > div { grid-template-columns: 110px minmax(0, 1fr); gap: 8px; }
  .review-support .evidence-list dd { white-space: normal; overflow-wrap: anywhere; }
  .review-support .receipt-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .review-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 20px;
    margin-bottom: 12px;
  }
  .review-header h1 { margin: 5px 0 4px; font-size: 22px; }
  .review-header .review-identity { font-family: "IBM Plex Mono", monospace; overflow-wrap: anywhere; }
  .review-header p { margin: 0; color: var(--pytxo-text-muted); font-size: 12px; }
  .review-screen .back {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    margin: 0 0 4px;
    padding: 0;
    border: 0;
    color: var(--pytxo-text-muted);
    background: transparent;
  }
  .review-actions { display: flex; gap: 8px; align-items: center; }
  .review-actions button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
  }
  .review-screen .danger-quiet { color: var(--state-refuted); border-color: var(--pytxo-line); background: var(--pytxo-surface-panel); }
  .review-status {
    display: flex;
    gap: 10px;
    align-items: center;
    min-height: 48px;
    padding: 10px 13px;
    border: 1px solid var(--pytxo-line);
    border-radius: 8px;
    background: var(--pytxo-surface-panel);
    color: var(--state-verified);
  }
  .review-status > div { display: grid; gap: 2px; }
  .review-status strong { font-size: 12px; }
  .review-status span { color: var(--pytxo-text-soft); font-size: 12px; }
  .review-status.state-stale,
  .review-status.state-verification_required,
  .review-status.state-review_failed,
  .review-status.state-recovery_required { color: var(--state-attention); border-color: var(--pytxo-line); background: var(--pytxo-surface-panel); }
  .review-status.state-applied { color: var(--state-verified); }
  .action-explanation, .action-notice, .action-error {
    margin: 8px 0 0;
    font-size: 12px;
  }
  .action-explanation { color: var(--state-attention); }
  .action-notice { color: var(--state-verified); }
  .action-error { color: var(--state-refuted); }
  .review-screen .review-grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 12px;
    margin-top: 12px;
    align-items: start;
  }
  .review-screen .review-grid > .panel { min-width: 0; padding: 0; overflow: hidden; }
  .files-panel { grid-column: 1; }
  .review-support { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); min-width: 0; gap: 12px; align-items: start; }
  .review-support > .panel { padding: 0; min-height: 0; }
  .attempts-panel { grid-column: 1 / -1; }
  @container run-review (max-width: 800px) {
    .review-support { grid-template-columns: minmax(0, 1fr); }
  }
  .panel-title {
    padding: 13px 15px 11px;
    border-bottom: 1px solid var(--pytxo-line-soft);
  }
  .panel-title .eyebrow, .panel-title h2 { margin: 0; }
  .review-screen .panel-title h2 { margin: 4px 0 0; font-size: 14px; color: var(--pytxo-text-strong); }
  .evidence-list { display: grid; margin: 0; }
  .evidence-list > div {
    display: grid;
    grid-template-columns: minmax(130px, .75fr) minmax(0, 1.25fr);
    gap: 10px;
    padding: 8px 15px;
    border-bottom: 1px solid var(--pytxo-line-soft);
  }
  .evidence-list dt { display: flex; gap: 6px; align-items: center; color: var(--pytxo-text-muted); font-size: 12px; }
  .evidence-list dd {
    margin: 0;
    overflow: hidden;
    color: var(--pytxo-text-body);
    font: 12px/1.4 "IBM Plex Mono", monospace;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .receipt-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1px; background: var(--pytxo-line-soft); }
  .evidence-list dd.candidate-checks { white-space: normal; overflow-wrap: anywhere; }
  .candidate-checks ul { margin: 6px 0; padding-left: 16px; }
  .receipt-grid > div { display: grid; grid-template-columns: auto 1fr; gap: 3px 7px; padding: 10px 12px; background: var(--pytxo-surface-panel); }
  .receipt-grid strong { font-size: 12px; }
  .receipt-grid small { grid-column: 2; color: var(--pytxo-text-muted); font: 12px/1.3 "IBM Plex Mono", monospace; }
  .receipt-dot { width: 7px; height: 7px; margin-top: 4px; border-radius: 50%; background: var(--state-unknown); }
  .receipt-dot.enforced { background: var(--live); }
  .receipt-dot.advisory { background: var(--pytxo-text-muted); }
  .task-list { display: grid; }
  .task-list > div {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    gap: 9px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--pytxo-line-soft);
  }
  .task-list > div > span { align-self: start; padding: 3px 5px; border-radius: 4px; color: var(--pytxo-text-soft); background: var(--pytxo-surface-raised); font: 600 9px/1 "IBM Plex Mono", monospace; }
  .task-list p { display: grid; gap: 2px; margin: 0; min-width: 0; }
  .task-list strong { font-size: 12px; overflow-wrap: anywhere; }
  .task-list small, .task-list em { color: var(--pytxo-text-muted); font: 12px/1.35 "IBM Plex Mono", monospace; overflow-wrap: anywhere; }
  .task-list em { grid-column: 2; }
  .files-title { display: flex; justify-content: space-between; gap: 12px; align-items: center; }
  .summary { display: flex; gap: 6px; flex-wrap: wrap; justify-content: flex-end; }
  .summary span { padding: 4px 6px; border-radius: 4px; background: var(--pytxo-surface-raised); font: 600 9px/1 "IBM Plex Mono", monospace; text-transform: uppercase; }
  .summary .add { color: var(--state-verified); } .summary .modify { color: var(--state-attention); } .summary .delete { color: var(--state-refuted); }
  .file-list { display: grid; }
  .file-row {
    border-bottom: 1px solid var(--pytxo-line-soft);
  }
  .file-row:last-child { border-bottom: 0; }
  .file-row.add { color: var(--state-verified); } .file-row.modify { color: var(--state-attention); } .file-row.delete { color: var(--state-refuted); }
  .file-row > header {
    display: grid;
    grid-template-columns: auto auto minmax(0, 1fr) auto auto;
    gap: 10px;
    align-items: center;
    padding: 10px 14px;
  }
  .file-row .kind { font: 600 11px/1 "IBM Plex Mono", monospace; text-transform: uppercase; }
  .file-row header > div { display: grid; gap: 2px; min-width: 0; }
  .file-row strong { color: var(--pytxo-text-strong); font: 12px/1.4 "IBM Plex Mono", monospace; overflow-wrap: anywhere; white-space: normal; }
  .file-row small { color: var(--pytxo-text-muted); font-size: 12px; overflow-wrap: anywhere; }
  .inspect-file { padding: 5px 8px; border-color: var(--pytxo-line); color: var(--pytxo-text-soft); background: var(--pytxo-surface-raised); font: 600 9px/1 "IBM Plex Mono", monospace; text-transform: uppercase; }
  .inspect-file.selected { border-color: var(--pytxo-text-muted); color: var(--pytxo-text-strong); background: var(--pytxo-surface-active); }
  .exact-diff { display: grid; grid-template-columns: 1fr 1fr; border-top: 1px solid var(--pytxo-line-soft); background: var(--pytxo-surface-input); }
  .exact-diff.single { grid-template-columns: 1fr; }
  .diff-side { min-width: 0; border-right: 1px solid var(--pytxo-line-soft); }
  .diff-side:last-child { border-right: 0; }
  .diff-label { display: flex; justify-content: space-between; gap: 10px; padding: 7px 10px; border-bottom: 1px solid var(--pytxo-line-soft); background: var(--pytxo-surface-raised); }
  .diff-label span { color: var(--pytxo-text-muted); font: 12px/1.4 "IBM Plex Mono", monospace; }
  .text-content, .binary-content { min-height: 74px; max-height: 320px; margin: 0; padding: 10px; overflow: auto; color: var(--pytxo-text-body); background: transparent; font: 12px/1.55 "IBM Plex Mono", monospace; white-space: pre-wrap; overflow-wrap: anywhere; }
  .before .text-content { background: color-mix(in oklab, var(--state-refuted) 7%, transparent); }
  .after .text-content { background: color-mix(in oklab, var(--state-verified) 7%, transparent); }
  .binary-label, .content-loading, .content-error { margin: 0; padding: 9px 10px 0; color: var(--pytxo-text-muted); font-size: 12px; }
  .content-error { color: var(--state-refuted); }
  .expand-content { margin: 0 10px 10px; color: var(--pytxo-text-strong); border-color: var(--pytxo-line); background: var(--pytxo-surface-raised); }
  .digest-details { padding: 8px 14px 10px; border-top: 1px solid var(--pytxo-line-soft); color: var(--pytxo-text-muted); }
  .digest-details summary { cursor: pointer; font-size: 12px; text-transform: uppercase; }
  .digest-details dl { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; margin: 8px 0 0; }
  .file-row dl div { min-width: 0; }
  .file-row dt { color: var(--pytxo-text-muted); font-size: 12px; text-transform: uppercase; }
  .file-row dd { margin: 2px 0 0; color: var(--pytxo-text-soft); font: 12px/1.4 "IBM Plex Mono", monospace; overflow-wrap: anywhere; }
  .bytes { color: var(--pytxo-text-muted); font: 12px/1 "IBM Plex Mono", monospace; }
  .attempt { display: flex; gap: 10px; padding: 12px 14px; border-bottom: 1px solid var(--pytxo-line-soft); }
  .attempt p { display: grid; min-width: 0; gap: 2px; margin: 0; overflow-wrap: anywhere; }
  .attempt strong { font-size: 12px; }
  .attempt span { color: var(--pytxo-text-soft); font-size: 12px; }
  .attempt small { color: var(--pytxo-text-muted); font: 12px/1.35 "IBM Plex Mono", monospace; }
  .attempt .audit-code { color: var(--pytxo-text-soft); text-transform: uppercase; letter-spacing: .04em; }
  .attempt .audit-code code { color: currentColor; font: inherit; text-transform: none; }
  .attempt.failure { color: var(--state-refuted); } .attempt.success { color: var(--state-verified); }
  .review-loading { display: grid; grid-template-columns: repeat(3, 1fr); gap: 12px; margin-top: 12px; }
  .review-loading span { height: 180px; border-radius: 8px; background: var(--pytxo-surface-panel); animation: review-pulse 1.2s ease-in-out infinite alternate; }
  .unavailable { display: flex; gap: 10px; margin-top: 12px; }
  .unavailable h2, .unavailable p { margin: 0; }
  .dialog-backdrop { position: fixed; inset: 0; z-index: 60; display: grid; place-items: center; padding: 20px; background: rgba(4, 6, 8, .72); }
  .confirm-dialog { display: grid; grid-template-columns: auto 1fr; gap: 12px; width: min(440px, 100%); padding: 18px; border: 1px solid var(--pytxo-line); border-radius: 10px; background: var(--pytxo-surface-panel); box-shadow: 0 20px 70px rgba(0,0,0,.45); color: var(--state-refuted); }
  .confirm-dialog h2 { margin: 0; color: var(--pytxo-text-strong); font-size: 16px; }
  .confirm-dialog p { margin: 5px 0 0; color: var(--pytxo-text-soft); font-size: 12px; line-height: 1.5; }
  .confirm-dialog code { display: block; overflow-wrap: anywhere; margin-top: 8px; color: var(--pytxo-text-soft); font: 10px/1.45 "IBM Plex Mono", monospace; }
  .confirm-dialog.apply-dialog { border-color: var(--pytxo-line); color: var(--state-verified); }
  .dialog-actions { grid-column: 1 / -1; display: flex; justify-content: flex-end; gap: 8px; margin-top: 8px; }
  .dialog-actions .confirm-apply { border-color: var(--pytxo-text-strong); color: var(--pytxo-surface-shell); background: var(--pytxo-text-strong); }
  .dialog-actions .confirm-apply:hover:not(:disabled) { background: var(--pytxo-text-body); }
  .dialog-actions .danger { border-color: var(--state-refuted); color: var(--state-refuted); background: color-mix(in oklab, var(--state-refuted) 7%, var(--pytxo-surface-panel)); }
  .dialog-actions .danger:hover:not(:disabled) { border-color: var(--state-refuted); background: color-mix(in oklab, var(--state-refuted) 12%, var(--pytxo-surface-panel)); }
  @keyframes review-spin { to { transform: rotate(360deg); } }
  @keyframes review-pulse { to { background: var(--pytxo-surface-raised); } }
  .review-spinner { animation: review-spin .9s linear infinite; }
  :global(.review-screen button:focus-visible), .confirm-dialog button:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  @media (max-width: 1180px) {
    .review-screen .review-grid { grid-template-columns: 1fr; }
    .files-panel, .attempts-panel { grid-column: auto; }
    .evidence-list > div { grid-template-columns: 1fr; gap: 3px; }
    .evidence-list dd {
      overflow: visible;
      overflow-wrap: anywhere;
      text-overflow: clip;
      white-space: normal;
    }
  }
  @media (max-width: 760px) {
    .review-header { display: grid; }
    .review-actions { flex-wrap: wrap; }
    .review-actions button { flex: 1 1 auto; }
    .receipt-grid { grid-template-columns: 1fr; }
    .file-row > header { grid-template-columns: auto minmax(0, 1fr) auto; }
    .file-row header > div { grid-column: 1 / -1; grid-row: 2; }
    .file-row .bytes { display: none; }
    .exact-diff { grid-template-columns: 1fr; }
    .diff-side { border-right: 0; border-bottom: 1px solid var(--pytxo-line-soft); }
    .diff-side:last-child { border-bottom: 0; }
    .digest-details dl { grid-template-columns: 1fr; }
    .task-list > div { grid-template-columns: auto minmax(0, 1fr); }
    .task-list em { grid-column: 2; }
  }
  @container run-review (max-width: 960px) {
    .review-screen .review-grid { grid-template-columns: 1fr; }
    .files-panel, .attempts-panel { grid-column: auto; }
  }
  @container run-review (max-width: 680px) {
    .review-header { display: grid; }
    .review-actions { flex-wrap: wrap; }
    .review-actions button { flex: 1 1 auto; }
    .file-row > header { grid-template-columns: auto minmax(0, 1fr) auto; }
    .file-row header > div { grid-column: 1 / -1; grid-row: 2; }
    .file-row .bytes { display: none; }
    .exact-diff { grid-template-columns: 1fr; }
    .digest-details dl { grid-template-columns: 1fr; }
  }
  @media (prefers-reduced-motion: reduce) {
    .review-spinner, .review-loading span { animation: none; }
  }
</style>
