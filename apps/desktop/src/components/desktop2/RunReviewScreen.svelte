<script lang="ts">
  import { withPreviewsHidden } from "../../lib/preview-overlay";
  import { onMount, tick } from "svelte";
  import IconAlertTriangle from "@tabler/icons-svelte/icons/alert-triangle";
  import IconArrowLeft from "@tabler/icons-svelte/icons/arrow-left";
  import IconCheck from "@tabler/icons-svelte/icons/check";
  import IconFileText from "@tabler/icons-svelte/icons/file-text";
  import IconFingerprint from "@tabler/icons-svelte/icons/fingerprint";
  import IconLoader2 from "@tabler/icons-svelte/icons/loader-2";
  import IconRefresh from "@tabler/icons-svelte/icons/refresh";
  import IconShieldCheck from "@tabler/icons-svelte/icons/shield-check";
  import IconTrash from "@tabler/icons-svelte/icons/trash";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import { workbenchSelection, rememberWorkbenchSelection } from "../../lib/workbench-selection";
  import { recordedCliFor, recordedWorkerLabel, reviewPresentation } from "../../lib/review-state";
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
    onChanged,
    onWork,
  }: {
    backend: DesktopBackend;
    run: RunDto;
    domainId: string | null;
    onBack: () => void;
    onChanged: () => void | Promise<void>;
    onWork: () => void;
  } = $props();

  let contentWidth = $state(0);
  let mapPreference = $state<string | null>(null);
  let evidenceDisclosure = $state<HTMLDetailsElement>();
  let fileNavigation = $state<HTMLDivElement>();
  const mapExpanded = $derived(mapPreference === "map" || (mapPreference !== "code" && contentWidth >= 900));
  function toggleCandidateMap(event: MouseEvent) {
    event.preventDefault();
    mapPreference = mapExpanded ? "code" : "map";
    try { localStorage.setItem("pytxo-review-composition-v1", mapPreference); } catch { /* Session choice still works without storage. */ }
  }
  async function revealChecks() {
    if (!evidenceDisclosure) return;
    evidenceDisclosure.open = true;
    await tick();
    const checks = evidenceDisclosure.querySelector<HTMLElement>(".candidate-checks");
    checks?.focus();
    checks?.scrollIntoView({ block: "nearest" });
  }
  function revealFileList() {
    fileNavigation?.querySelector<HTMLButtonElement>("button")?.focus();
    fileNavigation?.scrollIntoView({ block: "nearest" });
  }

  let missionTitle = $state<string | null>(null);
  $effect(() => {
    const id = run.id; const domain = domainId;
    missionTitle = null;
    let current = true;
    backend.flowHistory().then(records => {
      if (current) missionTitle = records.find(record => record.dispatched_run_id === id && record.domain_id === domain)?.title ?? null;
    }).catch(() => {});
    return () => { current = false; };
  });
  let review = $state<RunReviewDto | null>(null);
  let agents = $state<AgentDto[]>([]);
  let loading = $state(true);
  let actionPending = $state(false);
  let error = $state("");
  let notice = $state("");
  let confirmApply = $state(false);
  let confirmedPackageDigest = $state<string | null>(null);
  let reviewLoad = 0;
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
  const selectedPreparedFile = $derived(manifest?.files.find(file => file.path === selectedPreparedPath) ?? null);
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
    const request = ++reviewLoad;
    loading = true;
    error = "";
    try {
      const [nextReview, nextAgents] = await Promise.all([
        backend.runReview(run.id, domainId),
        backend.listAgents(run.id, domainId),
      ]);
      if (request !== reviewLoad) return;
      const invalidatedConfirmation = !!confirmedPackageDigest && confirmedPackageDigest !== nextReview.prepared_digest;
      if (invalidatedConfirmation) {
        confirmApply = false;
        confirmedPackageDigest = null;
        notice = "The candidate changed. Review the current changes and checks before Apply.";
      }
      review = nextReview;
      agents = nextAgents;
      if (invalidatedConfirmation) {
        await tick();
        primaryTrigger?.focus();
      }
      const remembered = workbenchSelection(run.id, domainId);
      const files = review?.prepared_manifest?.files ?? [];
      const firstFile = files.find(file => file.path === remembered.path)
        ?? files.find(file => file.task_id === remembered.taskId) ?? files[0];
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
      if (request === reviewLoad) error = cause instanceof Error ? cause.message : String(cause);
    } finally {
      if (request === reviewLoad) loading = false;
    }
  }

  async function runPrimaryAction(expectedPackageDigest?: string) {
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
        if (!expectedPackageDigest) throw new Error("Review and confirm the current candidate before Apply.");
        await backend.applyRunChanges(run.id, domainId, expectedPackageDigest);
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
      const actionError = cause instanceof Error ? cause.message : String(cause);
      try {
        await onChanged();
      } catch {
        /* Preserve the original action error; the immediate domain event is a second refresh path. */
      }
      await loadReview();
      // A refusal that left the review stale is explained by its status; the
      // runner's own wording would only repeat it in internal terms.
      error = presentation.state === "stale" ? "" : actionError;
    } finally {
      actionPending = false;
    }
  }

  async function requestPrimaryAction() {
    if (
      presentation.primaryAction === "apply" ||
      presentation.primaryAction === "retry"
    ) {
      if (!presentation.applyAllowed || !allAgentsSucceeded || actionPending || loading) return;
      if (!review?.prepared_digest || review.run_id !== run.id || manifest?.run_id !== run.id || manifest.package_digest !== review.prepared_digest) {
        error = "The displayed review identity is incomplete. Reload the review before Apply.";
        return;
      }
      confirmedPackageDigest = review.prepared_digest;
      await withPreviewsHidden(() => { confirmApply = true; });
      await tick();
      cancelApplyButton?.focus();
      return;
    }
    await runPrimaryAction();
  }

  async function applyExactPackage() {
    const expectedPackageDigest = confirmedPackageDigest;
    confirmApply = false;
    confirmedPackageDigest = null;
    if (expectedPackageDigest) await runPrimaryAction(expectedPackageDigest);
  }

  async function closeApplyDialog() {
    confirmApply = false;
    confirmedPackageDigest = null;
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
    const context = workbenchSelection(run.id, domainId);
    const recordedOwner = tasks.some(task => task.task_id === file.task_id) ? file.task_id : context.taskId;
    rememberWorkbenchSelection(run.id, domainId, { path: file.path, taskId: force && context.taskId ? context.taskId : recordedOwner });
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
    try { mapPreference = localStorage.getItem("pytxo-review-composition-v1"); } catch { /* Default follows content width. */ }
    let disposed = false;
    let unsubscribe: (() => void) | undefined;
    void backend.onDomainChanged((event) => {
      if (!disposed && !actionPending && event.domain_id === (domainId ?? run.domain_id)
        && event.entity_kind === "contract" && (event.entity_id === run.id || event.entity_id === "*")) {
        void loadReview();
      }
    }).then((stop) => { if (disposed) stop(); else unsubscribe = stop; })
      .catch(() => { /* Core still rejects stale authorization if notifications are unavailable. */ });
    void loadReview({ focus: true });
    return () => {
      disposed = true;
      reviewLoad += 1;
      contentSelection += 1;
      cancelQueuedContentReads();
      unsubscribe?.();
    };
  });
</script>

<section class="screen review-screen" bind:clientWidth={contentWidth} aria-labelledby="run-review-title">
  <section class="review-decision" aria-label="Review decision">
  <header class="review-header">
    <div>
      <button class="back" bind:this={backButton} onclick={onWork}>
        <IconArrowLeft size={15} /> Back to Work
      </button>
      <span class="review-mode">Review changes</span>
      <h1 id="run-review-title">{missionTitle || `Changes in ${run.repo_root.split(/[\\/]/).pop()}`}</h1>
      <div class="review-context">
        <span class="review-destination" title={run.repo_root}>Apply destination: <strong>{run.repo_root}</strong></span>
        <details class="run-details">
          <summary>Run details</summary>
          <code>{run.id}</code>
        </details>
      </div>
    </div>

  </header>


  </section>

  {#if loading}
    <div class="review-loading" aria-label="Loading review">
      <span></span><span></span><span></span>
    </div>
  {:else if review && manifest}
    <div class="review-scroll">
    <div class="candidate-workspace">
    <div class="speculative-workspace">
      <details class="candidate-overview" open={mapExpanded}>
        <summary onclick={toggleCandidateMap}><span class="candidate-symbol" aria-hidden="true"><IconFileText size={18} /></span><strong>{mapExpanded ? "Focus on code" : "Show candidate map"}</strong><span>{manifest.files.length} prepared file{manifest.files.length === 1 ? "" : "s"} · one exact candidate</span><span class="verification-summary" class:checks-passed={candidatePassed}>{candidatePassed ? "Combined checks: passed" : "Verification not established"}</span></summary>
        <div class="candidate-context" aria-label="Candidate relationships">
          <div class="formation-files">
            <span class="map-label">Prepared files</span>
            {#each manifest.files.slice(0, 3) as file}
              <button class:map-selected={selectedPreparedPath === file.path} aria-pressed={selectedPreparedPath === file.path} title={file.path} aria-label={`Compare ${file.path}`} onclick={() => void selectPreparedFile(file)}><span class="map-file-kind">{file.kind === "add" ? "A" : file.kind === "delete" ? "D" : "M"}</span><span>{file.path.split("/").pop()}</span></button>
            {/each}
            {#if manifest.files.length > 3}<button class="more-files" onclick={revealFileList}>+{manifest.files.length - 3} more · Browse all files</button>{/if}
          </div>
          <svg class="convergence" viewBox="0 0 80 120" preserveAspectRatio="none" aria-hidden="true">
            {#each manifest.files.slice(0, 3) as file, index}<path d={`M 0 ${20 + index * 40} C 40 ${20 + index * 40}, 35 60, 80 60`} class:selected={selectedPreparedPath === file.path} />{/each}
          </svg>
          <details class="candidate-node"><summary><IconFileText size={20}/><span>Exact candidate<small class="identity-peek" title={review.prepared_digest ?? "Identity unavailable"}>{review.prepared_digest?.slice(0, 18) ?? "Identity unavailable"}</small><small>{presentation.state === "applied" ? "Apply recorded" : "No confirmed Apply"}</small></span></summary><code>{review.prepared_digest ?? "Identity unavailable"}</code></details>
          <span class="stage-connection" aria-hidden="true">→</span>
          <button class="verification-node" onclick={() => void revealChecks()}><span class="map-label">Recorded verification</span><strong>{candidatePassed ? "Required checks passed" : "Verification not established"}</strong><span>{candidateEvidence?.checks.length ?? 0} recorded {(candidateEvidence?.checks.length ?? 0) === 1 ? "command" : "commands"} · View checks</span></button>
          <div class="map-boundary"><span class="decision-aperture" aria-hidden="true"></span><details class="destination-node"><summary><span class="map-label">Apply destination</span><strong>{run.repo_root.split(/[\\/]/).pop()}</strong><span>Read-only target details</span></summary><code>{run.repo_root}</code><p>Apply uses this run's repository. Inspecting this target makes no changes.</p></details></div>
        </div>
        <div class="map-caption">Candidate relationships, not code dependencies.<button onclick={() => document.getElementById("review-apply-decision")?.scrollIntoView({ block: "nearest" })}>{presentation.state === "applied" ? "View recorded outcome" : "Review the decision at the Apply boundary"} ↓</button></div>
      </details>
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
          <div class="file-navigation" bind:this={fileNavigation} role="group" aria-label="Prepared file navigation">
          {#each manifest.files as file (file.path)}
            {@const cli = recordedCliFor(agents, run, file.task_id)}
            <button class={`file-row ${file.kind}`} class:chosen={selectedPreparedPath === file.path}
              aria-label={`Inspect exact content for ${file.path}`} aria-pressed={selectedPreparedPath === file.path}
              title={file.path} onclick={() => void selectPreparedFile(file)}>
              <span class="file-kind">{file.kind === "add" ? "A" : file.kind === "delete" ? "D" : "M"}</span>
              <span class="file-name"><strong>{file.path.split("/").pop()}</strong><small>{file.path.includes("/") ? file.path.slice(0, file.path.lastIndexOf("/")) : "Repository root"}</small>{#if cli}<small class="file-cli">by {cli}</small>{/if}</span>
            </button>
          {/each}
          </div>
          <div class="file-content">
            {#if selectedPreparedFile}
              {@const file = selectedPreparedFile}
              {@const cli = recordedCliFor(agents, run, file.task_id)}
              <div class="comparison-context"><strong>{file.path.split("/").pop()}</strong><span>{cli ? `Prepared by ${cli} · ` : ""}Exact-content comparison · no inferred line changes</span></div>
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

              <details class="digest-details">
                <summary>Digests and mode metadata</summary>
                <p>{file.task_id} · {file.agent_id}</p>
                <dl>
                  <div><dt>Before SHA-256</dt><dd>{file.before_sha256 ?? "—"}</dd></div>
                  <div><dt>After SHA-256</dt><dd>{file.after_sha256 ?? "—"}</dd></div>
                  <div><dt>Mode</dt><dd>{file.before_mode ?? "portable"} → {file.after_mode ?? "portable"}</dd></div>
                </dl>
              </details>
            {/if}
          </div>
        </div>
      </article>


  {#if notice}<p class="action-notice" aria-live="polite">{notice}</p>{/if}
  {#if error}<p class="action-error" role="alert">{error}</p>{/if}


    </div>
    </div>
    </div>
    <details class="technical-evidence" bind:this={evidenceDisclosure}><summary>Commands, identity &amp; technical evidence</summary><aside class="review-support" aria-label="Review evidence and ownership">
      <article class="panel evidence-panel">
        <div class="panel-title"><p class="eyebrow">Immutable package</p><h2>Review evidence</h2></div>
        <dl class="evidence-list">
          <div><dt><IconShieldCheck size={14} /> Root</dt><dd>{run.repo_root}</dd></div>
          <div><dt>Profile</dt><dd>{review.enforcement.run.effective_profile}</dd></div>
          <div><dt>Paths</dt><dd>{manifest.summary.added + manifest.summary.modified + manifest.summary.deleted} · {manifest.summary.added} added · {manifest.summary.modified} modified · {manifest.summary.deleted} deleted</dd></div>
          <div><dt>State</dt><dd>{presentation.state}</dd></div>
          <div><dt>Combined candidate checks</dt><dd class="candidate-checks" tabindex="-1">
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
    </details>
    </div>
  {:else}
    <article class="panel unavailable">
      <IconAlertTriangle size={20} />
      <div><h2>Prepared package unavailable</h2><p>{error || "This run has no immutable review manifest."}</p></div>
    </article>
  {/if}
  <footer id="review-apply-decision" class:ready-decision={presentation.state === "ready" && !applyDisabledReason} class="decision-bar repository-boundary" class:confirmed={presentation.state === "applied"} aria-label="Decision">
    <div class="decision-main">
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
    {:else if presentation.state === "ready" || presentation.state === "applied"}
      <IconShieldCheck size={17} />
    {:else}
      <IconFileText size={17} />
    {/if}
    <div>
      <strong>{presentation.title}</strong>
      {#if presentation.state === "ready" && !applyDisabledReason}<details class="eligibility-detail"><summary>Why Apply is available</summary><span>{presentation.detail}</span></details>{:else}<span id={applyDisabledReason === presentation.detail ? "apply-disabled-reason" : undefined}>{presentation.detail}</span>{/if}
    </div>
  </div>

    <div class="decision-transition">
      <details class="decision-identity package-identity"><summary>Exact candidate</summary><code>{review?.prepared_digest ?? "Identity unavailable"}</code></details>
      <span class="decision-aperture" aria-hidden="true"></span>
      <div><strong>Canonical repository · {run.repo_root.split(/[\\/]/).pop()}</strong><span>{presentation.state === "applied" ? "Apply recorded" : presentation.state === "applying" ? "Apply in progress · outcome not confirmed" : "No confirmed Apply"}</span></div>
      <!-- A blocked reason equal to the decision copy is already shown above; show any other reason once, beside Apply. -->
      {#if applyDisabledReason && applyDisabledReason !== presentation.detail}
        <p class="boundary-reason blocked" id="apply-disabled-reason">{applyDisabledReason}</p>
      {:else}
        <p class="boundary-reason">{presentation.state === "applied" ? "Recorded outcome, not a live filesystem check." : "Apply writes this reviewed candidate to the destination."}</p>
      {/if}
    </div>
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
  </footer>
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
        {#if confirmedPackageDigest}
          <code>{confirmedPackageDigest}</code>
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

  .candidate-workspace{display:grid;grid-template-columns:minmax(0,1fr) 220px;border:1px solid var(--pytxo-line);min-width:0}
  .speculative-workspace{min-width:0}.candidate-formation{padding:16px 20px;border-bottom:1px solid var(--pytxo-line-soft)}.candidate-formation>span{font-size:10px;text-transform:uppercase;letter-spacing:.07em;color:var(--pytxo-text-muted)}.candidate-formation>div{display:flex;align-items:center;gap:14px;margin-top:14px;flex-wrap:wrap}.file-stack{padding:10px 12px;border:1px solid var(--pytxo-line);border-radius:3px;font:11px "IBM Plex Mono",monospace}.candidate-formation strong{font-size:13px}.candidate-formation small{color:var(--pytxo-text-muted);font-size:11px}
  .review-order{display:grid;grid-template-columns:repeat(4,minmax(0,1fr));padding:0;list-style:none;border-block:1px solid var(--pytxo-line);margin:0}.review-order span{font-size:10px;text-transform:uppercase;letter-spacing:.04em;color:var(--pytxo-text-muted)}.review-order strong{font-size:12px;font-weight:500}
  .checks-passed{color:var(--state-verified)}

  .decision-bar{position:sticky;bottom:0;z-index:5;display:flex;align-items:center;justify-content:space-between;gap:16px;padding:16px 0;background:var(--pytxo-surface-shell);border-top:1px solid var(--pytxo-line)}.decision-bar>div:first-child{display:grid;gap:6px;min-width:0;font-size:12px;overflow-wrap:anywhere}.decision-bar span{color:var(--pytxo-text-muted);font-size:11px}

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
  .package-identity { min-width: 0; max-width: 100%; color: var(--pytxo-text-muted); font-size: 12px; }
  .package-identity summary { display: flex; align-items: center; gap: 6px; cursor: pointer; min-height: 32px; list-style: none; }
  .package-identity summary::before { content: "›"; font-size: 16px; }
  .package-identity[open] summary::before { transform: rotate(90deg); }
  .package-identity summary:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .package-identity code { display: block; max-width: 44ch; padding-top: 6px; color: var(--pytxo-text-body); overflow-wrap: anywhere; font: 12px "IBM Plex Mono", monospace; }
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
  .review-header p { margin: 0; color: var(--pytxo-text-muted); font-size: 12px; }
  .review-context { display: flex; align-items: center; flex-wrap: wrap; gap: 5px 10px; min-height: 24px; color: var(--pytxo-text-muted); font-size: 12px; }
  .run-details { position: relative; }
  .run-details summary { cursor: pointer; color: var(--pytxo-text-muted); }
  .run-details summary:hover { color: var(--pytxo-text-body); }
  .run-details summary:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 2px; }
  .run-details code { display: block; max-width: min(460px, calc(100vw - 56px)); margin-top: 5px; padding: 6px 8px; border: 1px solid var(--pytxo-line-soft); border-radius: 4px; background: var(--pytxo-surface-input); color: var(--pytxo-text-body); font: 11px/1.4 "IBM Plex Mono", monospace; overflow-wrap: anywhere; }
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
    color: var(--pytxo-text-soft);
  }
  .review-status > div { display: grid; gap: 2px; }
  .review-status strong { font-size: 12px; }
  .review-status span { color: var(--pytxo-text-soft); font-size: 12px; }
  .review-status.state-stale,
  .review-status.state-verification_required,
  .review-status.state-review_failed,
  .review-status.state-recovery_required { color: var(--state-attention); border-color: var(--pytxo-line); background: var(--pytxo-surface-panel); }
  .review-status.state-ready,
  .review-status.state-applied { color: var(--state-verified); }
  .boundary-reason.blocked { color: var(--state-attention); }
  .action-notice, .action-error {
    margin: 8px 0 0;
    font-size: 12px;
  }
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
  .file-row strong { color: var(--pytxo-text-strong); font: 12px/1.4 "IBM Plex Mono", monospace; overflow-wrap: anywhere; white-space: normal; }
  .file-row small { color: var(--pytxo-text-muted); font-size: 12px; overflow-wrap: anywhere; }
  .file-row small.file-cli { display: block; color: var(--pytxo-text-soft); }
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
  .digest-details summary { cursor: pointer; font-size: 12px; }
  .digest-details dl { display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 8px; margin: 8px 0 0; }
  .file-content dl div { min-width: 0; }
  .file-content dt { color: var(--pytxo-text-muted); font-size: 12px; text-transform: uppercase; }
  .file-content dd { margin: 2px 0 0; color: var(--pytxo-text-soft); font: 12px/1.4 "IBM Plex Mono", monospace; overflow-wrap: anywhere; }
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
    .exact-diff { grid-template-columns: 1fr; }
    .digest-details dl { grid-template-columns: 1fr; }
  }
  @media (prefers-reduced-motion: reduce) {
    .review-spinner, .review-loading span { animation: none; }
  }

  .review-screen{gap:14px}.review-screen .review-grid{display:flex;flex-direction:column;align-items:stretch;gap:0}.review-support{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));gap:0}.review-screen .panel{border-radius:0;border:0}.review-header{padding-bottom:0}.review-status{margin-top:10px}.decision-evidence{margin-top:10px}.files-title{padding:12px 16px}
  .file-list{display:grid;grid-template-columns:240px minmax(0,1fr);min-height:280px;align-items:start}.file-navigation{max-height:430px;overflow:auto;border-right:1px solid var(--pytxo-line)}.file-row{display:block}.file-content{min-width:0}.file-content>.exact-diff{border-top:0;min-width:0}.diff-side .text-content{min-height:240px;max-height:380px}.file-content .digest-details dl{grid-template-columns:1fr}.review-header h1{font-size:26px}
  @media(max-width:1100px){.candidate-workspace{grid-template-columns:1fr}.review-support{grid-template-columns:1fr}.file-list{grid-template-columns:175px minmax(0,1fr)}}
  @media(max-width:760px){.review-order{grid-template-columns:repeat(2,minmax(0,1fr))}.file-list{display:block;min-height:0}.file-row{display:block}.file-content>.exact-diff{display:grid;grid-template-columns:1fr}.file-content>.digest-details{display:block}.decision-bar{flex-wrap:wrap;padding:10px 0}.decision-bar .review-actions{width:100%;justify-content:flex-start;gap:8px}.diff-side .text-content{min-height:100px}.candidate-formation small{display:none}}

  .review-header{align-items:center}.review-header h1{margin-top:6px}.review-header .back{min-height:26px;padding:0;border:0}.review-context{margin-top:4px}.review-order{margin-top:0}.candidate-workspace .review-status{margin:12px 16px}.candidate-workspace .decision-evidence{padding:0 16px 12px}.candidate-workspace .action-error,.candidate-workspace .action-notice{margin:8px 16px}.technical-evidence{border:1px solid var(--pytxo-line)}
  @container run-review (max-width:1000px){
    .candidate-workspace{grid-template-columns:1fr}.review-support{grid-template-columns:1fr}.file-list{display:block;min-height:0}.file-row{display:block}.file-content>.exact-diff{display:grid;grid-template-columns:1fr}.file-content>.digest-details{display:block}.review-order{grid-template-columns:repeat(2,minmax(0,1fr))}.decision-bar{flex-wrap:wrap}.decision-bar .review-actions{width:100%;justify-content:flex-start}
  }

  @container run-review (max-width:1000px){.file-navigation{max-height:240px;border-right:0;border-bottom:1px solid var(--pytxo-line)}.file-list{display:block}.file-content>.exact-diff{grid-template-columns:1fr}}

  /* Focused reading mode: the system recedes, the decision owns the aperture. */
  .review-screen { gap: 12px; padding-top: 18px; }
  .review-header { margin: 0; }.review-decision { padding-bottom: 12px; border: 0; }
  .review-header>div { width: 100%; }.review-header h1 { font-size: clamp(20px, 2vw, 28px); line-height: 1.25; margin: 5px 0 8px; overflow-wrap: anywhere; }
  .review-mode { display: block; color: var(--pytxo-text-muted); font-size: 11px; margin-top: 8px; }
  .candidate-workspace { display: block; }.candidate-overview { border-bottom: 1px solid var(--pytxo-line); }
  .candidate-overview>summary { padding: 14px 18px; display: flex; flex-wrap: wrap; align-items: center; gap: 12px; cursor: pointer; font-size: 12px; color: var(--pytxo-text-soft); }
  .candidate-overview>summary::after { content: "+"; margin-left: auto; font-size: 11px; }.candidate-overview[open]>summary::after { content: "−"; }
  .candidate-symbol { color: var(--pytxo-activity); font-size: 22px; }
  .file-list { grid-template-columns: 230px minmax(0,1fr); min-height: 260px; }
  .file-navigation { max-height: 60vh; background: var(--pytxo-surface-panel); padding: 6px; }
  .review-screen .file-row { display: flex; width: 100%; gap: 10px; padding: 12px 10px; border: 0; border-radius: 3px; text-align: left; background: transparent; align-items: center; justify-content: flex-start; text-transform: none; }
  .review-screen .file-row .file-name { flex: 1; min-width: 0; }
  .review-screen .file-row.chosen { background: var(--pytxo-surface-active); box-shadow: inset 2px 0 var(--pytxo-activity); }
  .file-kind { flex: none; font: 11px "IBM Plex Mono",monospace; color: var(--pytxo-text-soft); width: 22px; height: 24px; display: grid; place-items: center; border: 1px solid var(--pytxo-line); border-radius: 3px; }
  .file-name { display: grid; gap: 5px; min-width: 0; }.file-name strong { font-size: 13px; font-weight: 500; }.file-name small { display: block; font-size: 11px; color: var(--pytxo-text-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .comparison-context { display: flex; align-items: center; gap: 16px; padding: 10px 14px; border-bottom: 1px solid var(--pytxo-line); font-size: 11px; flex-wrap: wrap; }.comparison-context span { color: var(--pytxo-text-muted); }
  .diff-side.before,.diff-side.after,.diff-side .text-content { background: var(--pytxo-surface-input); }
  .diff-side .text-content { min-height: 200px; max-height: 52vh; font-size: 13px; line-height: 1.7; padding: 14px; tab-size: 2; }
  .technical-evidence>summary { padding: 12px 16px; cursor: pointer; font-size: 12px; color: var(--pytxo-text-muted); }
  .decision-transition { display: flex !important; flex-wrap: wrap; align-items: center; gap: 12px !important; }
  .decision-transition>div { display: grid; gap: 5px; }.decision-identity { position: relative; font-size: 12px; }.decision-identity summary { cursor: pointer; }.decision-identity code { position: absolute; bottom: 30px; left: 0; width: min(50ch,70vw); overflow-wrap: anywhere; padding: 12px; background: var(--pytxo-surface-raised); border: 1px solid var(--pytxo-line); }
  .decision-aperture { height: 38px; width: 3px; background: var(--pytxo-aperture); flex: none; }.boundary-reason { flex-basis: 100%; margin: 0; color: var(--pytxo-text-muted); font-size: 11px; max-width: 80ch; }
  .decision-bar.confirmed .decision-aperture { animation: confirmed-boundary 220ms ease-out; }.decision-bar.confirmed .decision-transition>div>span { color: var(--state-verified); }
  @keyframes confirmed-boundary { from { opacity: .3; } to { opacity: 1; } }
  .candidate-overview>summary:focus-visible,.technical-evidence>summary:focus-visible,.decision-identity>summary:focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: -2px; }
  @container run-review (max-width:1000px) { .file-list { display: grid; grid-template-columns: 180px minmax(0,1fr); }.file-navigation { max-height: 60vh; border-right: 1px solid var(--pytxo-line); }.file-content>.exact-diff { grid-template-columns: 1fr; }.decision-bar .review-actions { width: auto; } }
  @container run-review (max-width:640px) { .file-list { display: block; }.file-navigation { max-height: 150px; }.candidate-overview>summary { gap: 8px; }.comparison-context { display: grid; gap: 5px; }.decision-bar .review-actions { width: 100%; } }
  @media(max-height:700px) { .review-screen { padding-top: 10px; }.review-mode { margin-top: 0; }.review-header h1 { font-size: 20px; }.review-decision { padding-bottom: 0; }.review-context { font-size: 12px; }.candidate-overview>summary { padding-block: 9px; }.decision-bar { padding-block: 9px; } }
  @media(prefers-reduced-motion:reduce) { .decision-bar.confirmed .decision-aperture { animation: none; } }
  .verification-summary { margin-left: auto; }
  .decision-main { min-width: 0; flex: 1; }
  .decision-main .review-status { border: 0; padding: 0; margin: 0 0 10px; background: transparent; }
  .decision-main .review-status strong { font-size: 14px; }
  .decision-main .review-status span { font-size: 12px; }
  .decision-transition>div>strong { font-size: 13px; }
  .decision-bar .decision-transition>div>span,.boundary-reason { font-size: 12px; }
  .review-destination { overflow-wrap: anywhere; font-size: 13px; }
  .review-destination strong { font-weight: 500; color: var(--pytxo-text-strong); }
  @container run-review (max-width:800px) { }
  @container run-review (max-width:420px) { }
  @container run-review (max-width:640px) { .verification-summary { width: 100%; margin-left: 0; } }

  .review-screen { padding-top: 8px; gap: 8px; }
  .review-decision { padding-bottom: 4px; }
  .review-mode { display: none; }
  .review-header h1 { margin: 4px 0; }
  .candidate-overview>summary { padding: 9px 16px; }
  .candidate-context { display: grid; grid-template-columns: minmax(150px,1.1fr) 56px minmax(150px,1fr) 28px minmax(170px,1fr) minmax(180px,1fr); align-items: center; padding: 12px 16px; gap: 0; }
  .map-label { color: var(--pytxo-text-muted); font-size: 11px; font-weight: 500; white-space: nowrap; }
  .formation-files { display: grid; gap: 5px; min-width: 0; }
  .formation-files button { display: flex; justify-content: flex-start; gap: 8px; min-height: 28px; padding: 4px 8px; border: 1px solid var(--pytxo-line); border-radius: 3px; background: var(--pytxo-surface-panel); color: var(--pytxo-text-body); font-size: 12px; text-align: left; }
  .formation-files button>span:last-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .formation-files .map-selected { border-color: var(--pytxo-activity); background: var(--pytxo-surface-active); }
  .map-file-kind { font: 10px "IBM Plex Mono",monospace; color: var(--pytxo-activity); }
  .formation-files .more-files { border-color: transparent; background: transparent; color: var(--pytxo-text-soft); font-size: 11px; }
  .convergence { width: 100%; height: 112px; fill: none; stroke: var(--pytxo-line); stroke-width: 1.5; }
  .convergence path.selected { stroke: var(--pytxo-activity); }
  .candidate-node { border: 1px solid var(--pytxo-line); background: var(--pytxo-surface-panel); border-radius: 4px; padding: 12px; min-width: 0; }
  .candidate-node summary { display: flex; align-items: center; gap: 10px; cursor: pointer; font-size: 13px; }
  .candidate-node small { display: block; color: var(--pytxo-text-muted); margin-top: 6px; font-size: 11px; }
  .candidate-node code,.destination-node code { display: block; overflow-wrap: anywhere; margin-top: 10px; font-size: 11px; }
  .stage-connection { text-align: center; color: var(--pytxo-text-muted); }
  .candidate-context .verification-node { grid-template-columns: 1fr; align-items: start; min-width: 0; display: grid; gap: 7px; text-align: left; justify-content: start; padding: 12px; border: 0; border-radius: 0; background: transparent; color: var(--pytxo-text-body); }
  .verification-node strong { font-size: 13px; font-weight: 500; }
  .verification-node>span:last-child { font-size: 11px; color: var(--pytxo-text-soft); }
  .map-boundary { display: flex; gap: 14px; align-items: stretch; min-width: 0; margin-left: 12px; }
  .map-boundary .decision-aperture { height: auto; min-height: 84px; }
  .destination-node { min-width: 0; }
  .destination-node summary { display: grid; gap: 7px; cursor: pointer; font-size: 13px; }
  .destination-node summary>span:last-child,.destination-node p { font-size: 11px; color: var(--pytxo-text-soft); line-height: 1.5; }
  .map-caption { display: flex; flex-wrap: wrap; justify-content: space-between; gap: 8px; font-size: 11px; color: var(--pytxo-text-muted); padding: 0 16px 10px; }
  .map-caption button { border: 0; background: transparent; padding: 0; min-height: 24px; font-size: 11px; color: var(--pytxo-text-soft); text-underline-offset: 3px; }
  .candidate-context :is(button,summary):focus-visible { outline: 2px solid var(--pytxo-accent); outline-offset: 3px; }
  @container run-review (max-width:899px) {
    .candidate-context { grid-template-columns: 1fr 1fr; gap: 16px; }
    .convergence,.stage-connection { display: none; }
    .map-boundary { margin: 0; }
  }
  @container run-review (max-width:480px) { .candidate-context { grid-template-columns: 1fr; } }
  .review-screen { padding-top:8px; gap:8px; }
  .review-decision { padding-bottom:0; }
  .review-header h1 { margin-block:4px; }
  .review-mode { display:inline; margin-left:12px; }
  .candidate-overview>summary { padding:6px 12px; gap:8px; }
  .candidate-context { padding:4px 12px; }
  .formation-files { gap:3px; }
  .formation-files button { min-height:24px; padding:2px 6px; }
  .convergence { height:88px; }
  .candidate-node { padding:8px; }
  .candidate-context .verification-node { padding:8px; gap:4px; }
  .map-caption { padding-bottom:4px; }
  .map-boundary .decision-aperture { min-height:64px; }
  .files-title { padding:8px 12px; }
  .files-title .eyebrow { display:none; }
  .comparison-context { padding:6px 12px; }
  .decision-bar { padding-block:8px; gap:12px; }
  .decision-main .review-status { margin-bottom:6px; }
  .ready-decision .decision-main { display:flex; align-items:center; flex-wrap:wrap; gap:6px 18px; }
  .ready-decision .review-status { margin:0; }
  .ready-decision .boundary-reason { display:none; }
  .eligibility-detail { font-size:12px; color:var(--pytxo-text-soft); position:relative; }
  .eligibility-detail summary { cursor:pointer; }
  .eligibility-detail[open] span { position:absolute; bottom:24px; left:0; width:min(48ch,65vw); padding:12px; border:1px solid var(--pytxo-line); background:var(--pytxo-surface-raised); z-index:3; }
  @container run-review (min-width:900px) { .decision-bar { flex-wrap:nowrap; }.decision-bar .review-actions { width:auto; flex-shrink:0; } }

  :global(.desktop2) .review-screen.screen { padding-top:8px; padding-bottom:12px; }
  .decision-bar.ready-decision>.decision-main:first-child { display:flex; align-items:center; gap:12px; }
  .ready-decision .review-status { min-height:0; flex:none; }
  .ready-decision .decision-transition { flex:1; }
  .ready-decision .decision-transition>div { gap:2px; }
  .formation-files { grid-template-columns:minmax(0,1fr) auto; }
  .formation-files .map-label { grid-column:1; grid-row:1; }
  .formation-files .more-files { grid-column:2; grid-row:1; padding:0; min-height:20px; font-size:11px; }
  .formation-files button:not(.more-files) { grid-column:1 / -1; }
  .map-caption { padding:0 12px 3px; align-items:center; }
  .map-caption button { min-height:18px; }
  .review-screen .files-title { padding:5px 12px; margin:0; }
  .review-screen .files-title h2 { font-size:14px; }
  .review-screen .review-grid { margin-top:0; }
  .review-header .back { min-height:20px; }
  .review-header h1 { font-size:22px; }
  @container run-review (max-width:899px) { .formation-files { grid-template-columns:1fr; }.formation-files .more-files { grid-column:1;grid-row:auto; }.decision-bar.ready-decision>.decision-main:first-child { flex-wrap:wrap; } }

  /* Review reads as one code workspace, with the candidate map as its context. */
  .candidate-overview {
    overflow:hidden;
    border:1px solid color-mix(in srgb,var(--pytxo-line) 84%,var(--pytxo-activity));
    border-radius:8px;
    background:var(--pytxo-work-canvas);
  }
  .candidate-overview>summary { background:color-mix(in srgb,var(--pytxo-surface-panel) 72%,transparent); }
  .candidate-overview[open]>summary { border-bottom:1px solid var(--pytxo-line-soft); }
  .candidate-context { background:var(--pytxo-work-canvas); }
  .formation-files button { border-color:var(--pytxo-line-soft);background:var(--pytxo-work-node); }
  .formation-files button:hover { border-color:color-mix(in srgb,var(--pytxo-activity) 46%,var(--pytxo-line)); }
  .formation-files .map-selected {
    border-color:color-mix(in srgb,var(--pytxo-activity) 70%,var(--pytxo-line));
    background:color-mix(in srgb,var(--pytxo-work-node) 82%,var(--pytxo-activity));
    box-shadow:inset 0 0 0 1px color-mix(in srgb,var(--pytxo-activity) 22%,transparent);
  }
  .convergence { stroke:color-mix(in srgb,var(--pytxo-text-muted) 46%,var(--pytxo-line)); }
  .candidate-node { border-color:color-mix(in srgb,var(--pytxo-line) 84%,var(--pytxo-activity));background:var(--pytxo-work-node); }
  .candidate-context .verification-node { border-radius:4px;background:color-mix(in srgb,var(--pytxo-surface-panel) 62%,transparent); }
  .map-boundary { padding-left:12px;border-left:1px solid var(--pytxo-line-soft); }
  .review-screen .review-grid {
    overflow:hidden;
    border:1px solid var(--pytxo-line);
    border-radius:8px;
    background:var(--pytxo-surface-panel);
  }
  .review-screen .files-title { background:color-mix(in srgb,var(--pytxo-surface-panel) 76%,var(--pytxo-work-canvas));border-bottom:1px solid var(--pytxo-line); }
  .file-navigation { background:color-mix(in srgb,var(--pytxo-surface-panel) 88%,var(--pytxo-work-canvas)); }
  .review-screen .file-row { transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease),color var(--pytxo-motion-fast) var(--pytxo-motion-ease); }
  .review-screen .file-row:hover { background:var(--pytxo-surface-hover); }
  .review-screen .file-row.chosen {
    background:color-mix(in srgb,var(--pytxo-surface-active) 82%,var(--pytxo-activity));
    box-shadow:inset 2px 0 var(--pytxo-activity);
  }
  .comparison-context { background:color-mix(in srgb,var(--pytxo-surface-panel) 86%,var(--pytxo-work-canvas)); }
  .diff-side.before,.diff-side.after,.diff-side .text-content { background:var(--pytxo-code-surface); }
  .diff-side .text-content { color:var(--pytxo-text-body); }
  .decision-bar {
    padding-inline:14px;
    border:1px solid color-mix(in srgb,var(--pytxo-line) 84%,var(--pytxo-activity));
    border-radius:8px;
    background:color-mix(in srgb,var(--pytxo-surface-panel) 88%,var(--pytxo-work-canvas));
    box-shadow:0 14px 34px -30px color-mix(in srgb,var(--pytxo-text-strong) 32%,transparent);
  }
  .decision-aperture { width:4px;border-radius:2px; }

  @container run-review (max-width:899px) {
    .map-boundary { padding-left:0;border-left:0; }
  }

  /* Review is a fixed Work state: evidence scrolls while the decision stays visible. */
  .review-screen{display:flex;width:100%;max-width:none;height:100%;min-height:0;box-sizing:border-box;flex-direction:column;overflow:hidden}
  .review-decision{flex:0 0 auto}
  .review-scroll{flex:1;min-height:0;overflow:auto;overscroll-behavior:contain;scrollbar-gutter:stable}
  .candidate-workspace{overflow:visible}
  .review-loading,.review-error,.review-unavailable{min-height:0;flex:1;overflow:auto}
  .decision-bar{position:relative;bottom:auto;flex:0 0 auto;margin:0;padding-block:8px;background:var(--pytxo-surface-shell)}
  @media(max-height:800px){.review-screen{gap:6px}.decision-bar{padding-block:2px}.convergence{height:44px}.candidate-node{padding:6px}.map-boundary .decision-aperture{min-height:44px}.diff-side .text-content{min-height:176px}}
</style>
