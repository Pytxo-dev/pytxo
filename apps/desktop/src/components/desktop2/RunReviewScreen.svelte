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
  import { diffLines, diffStats, foldUnchanged } from "../../lib/line-diff";
  import type {
    AgentDto,
    EnforcementSurface,
    PreparedContentChunkDto,
    PreparedRunFile,
    RunDto,
    RunReviewDto,
  } from "../../lib/types";

  /** Last two folders of a deep path; the full path stays in the title and Show folder. */
  const compactPath = (path: string) => {
    const parts = path.split(/[\\/]/).filter(Boolean);
    return parts.length > 3 ? `…/${parts.slice(-2).join("/")}` : path;
  };

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

  // "changes" is a line diff computed from the same exact bytes; "exact" shows both sides whole.
  type ComparisonMode = "changes" | "exact";
  let comparisonMode = $state<ComparisonMode>("changes");
  let openFolds = $state<string[]>([]);
  function chooseComparison(mode: ComparisonMode) {
    comparisonMode = mode;
    try { localStorage.setItem("pytxo-review-comparison-v1", mode); } catch { /* Session choice still works without storage. */ }
  }
  const selectedDiff = $derived.by(() => {
    const file = selectedPreparedFile;
    if (!file) return null;
    const before = file.before_sha256 ? preparedContent[contentKey(file, "before")] : undefined;
    const after = file.after_sha256 ? preparedContent[contentKey(file, "after")] : undefined;
    const sides = [before, after].filter((side) => side !== undefined);
    if ((file.before_sha256 && !before) || (file.after_sha256 && !after) || sides.some((side) => side?.loading && !side.complete)) return { status: "loading" as const };
    if (sides.some((side) => side?.error)) return { status: "error" as const };
    if (sides.some((side) => side?.binary)) return { status: "binary" as const };
    if (sides.some((side) => !side?.complete)) return { status: "partial" as const };
    const lines = diffLines(before ? textContent(before) : "", after ? textContent(after) : "");
    if (!lines) return { status: "too_large" as const };
    return { status: "ready" as const, rows: foldUnchanged(lines), ...diffStats(lines) };
  });
  function loadWholeFile(file: PreparedRunFile) {
    if (file.before_sha256) void loadPreparedContent(file, "before", true);
    if (file.after_sha256) void loadPreparedContent(file, "after", true);
  }
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

  // The Apply gate, left to right: each condition Apply waits on, in the order
  // it holds. ✓ is recorded, ○ is not known yet, ✗ is refuted, · is still running.
  type GateMark = "ok" | "open" | "fail" | "wait";
  const GATE_GLYPH: Record<GateMark, string> = { ok: "✓", open: "○", fail: "✗", wait: "·" };
  const agentsGate = $derived<GateMark>(allAgentsSucceeded ? "ok"
    : agents.some((agent) => agent.exit_code !== null && agent.exit_code !== 0) ? "fail" : "wait");
  const changeGate = $derived<GateMark>((manifest?.files.length ?? 0) > 0 ? "ok" : "fail");
  const checksGate = $derived<GateMark>(candidatePassed ? "ok" : candidateEvidence?.checks.some((check) => !check.passed) ? "fail" : "open");
  const projectGate = $derived<GateMark>(
    presentation.state === "applied" ? "ok"
    : presentation.state === "applying" ? "wait"
    : ["stale", "recovery_required", "retryable_failure", "review_failed"].includes(presentation.state) ? "fail"
    : "open");
  const projectGateText = $derived(
    presentation.state === "applied" ? "matched, written"
    : presentation.state === "applying" ? "writing…"
    : presentation.state === "stale" ? "changed since review"
    : projectGate === "fail" ? "Apply interrupted"
    : "checked again at Apply");
  const gateMarks = $derived([agentsGate, changeGate, checksGate, projectGate]);
  /** Signal flows through the wires only while Apply is actually available. */
  const gateOpen = $derived(presentation.state === "ready" && gateMarks.slice(0, 3).every((mark) => mark === "ok"));
  let landed = $state(false);
  let lastGateState: string | null = null;
  $effect(() => {
    const state = presentation.state;
    if (state === "applied" && lastGateState !== null && lastGateState !== "applied") {
      landed = true;
      setTimeout(() => (landed = false), 1800);
    }
    lastGateState = state;
  });
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
          ? "An earlier Apply was interrupted. Recover before applying again."
          : presentation.detail
        : !allAgentsSucceeded
          ? "Every agent must finish successfully before Apply."
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
        notice = "The changes were updated. Review them again before Apply.";
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
        notice = "Review refreshed against the current project files.";
      } else if (
        presentation.primaryAction === "apply" ||
        presentation.primaryAction === "retry"
      ) {
        if (!presentation.applyAllowed || !allAgentsSucceeded) return;
        if (!expectedPackageDigest) throw new Error("Review the current changes before Apply.");
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
        error = "This review did not load completely. Reload it before Apply.";
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
      notice = "Changes discarded.";
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
          throw new Error("This file changed while loading. Refresh the review.");
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
    try { if (localStorage.getItem("pytxo-review-comparison-v1") === "exact") comparisonMode = "exact"; } catch { /* Default shows changes. */ }
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
        <span class="review-destination" title={run.repo_root}>Applies to <strong>{compactPath(run.repo_root)}</strong></span>
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
        <summary onclick={toggleCandidateMap}><span class="candidate-symbol" aria-hidden="true">▣</span><strong>{mapExpanded ? "Hide summary" : "Show summary"}</strong><span>{manifest.files.length} file{manifest.files.length === 1 ? "" : "s"} changed</span><span class="gate-mini" aria-hidden="true">{#each gateMarks as mark}<i class={mark}>{GATE_GLYPH[mark]}</i>{/each}</span><span class="verification-summary" class:checks-passed={candidatePassed}>{candidatePassed ? "Checks passed" : "Not verified"}</span></summary>
        <div class="candidate-context" class:gate-open={gateOpen} class:landed aria-label="What this review contains">
          <div class="formation-files">
            <span class="map-label">changed files</span>
            {#each manifest.files.slice(0, 3) as file}
              <button class:map-selected={selectedPreparedPath === file.path} aria-pressed={selectedPreparedPath === file.path} title={file.path} aria-label={`Compare ${file.path}`} onclick={() => void selectPreparedFile(file)}><span class={`map-file-kind ${file.kind}`}>{file.kind === "add" ? "A" : file.kind === "delete" ? "D" : "M"}</span><span>{file.path.split("/").pop()}</span></button>
            {/each}
            {#if manifest.files.length > 3}<button class="more-files" onclick={revealFileList}>+{manifest.files.length - 3} more</button>{/if}
          </div>
          <svg class="convergence" viewBox="0 0 48 120" preserveAspectRatio="none" aria-hidden="true">
            {#each manifest.files.slice(0, 3) as file, index}<path d={`M 0 ${20 + index * 40} H 22 V 60 H 48`} class:selected={selectedPreparedPath === file.path} />{/each}
          </svg>
          <div class="gate" role="group" aria-label="Apply gate">
            <div class={`stage ${agentsGate}`}><span class="mark" aria-hidden="true">{GATE_GLYPH[agentsGate]}</span><span class="stage-text"><span class="map-label">agents</span><strong>{agents.filter((agent) => agent.exit_code === 0).length}/{agents.length} finished</strong></span></div>
            <span class="wire" class:lit={agentsGate === "ok"} aria-hidden="true"></span>
            <details class={`stage candidate-node ${changeGate}`}><summary><span class="mark" aria-hidden="true">{GATE_GLYPH[changeGate]}</span><span class="stage-text"><span class="map-label">Prepared change</span><strong>{manifest.files.length} file{manifest.files.length === 1 ? "" : "s"}</strong><small class="identity-peek" title={review.prepared_digest ?? "ID unavailable"}>{review.prepared_digest?.slice(0, 14) ?? "ID unavailable"}</small></span></summary><div class="node-pop"><code>{review.prepared_digest ?? "ID unavailable"}</code></div></details>
            <span class="wire" class:lit={changeGate === "ok"} aria-hidden="true"></span>
            <button class={`stage verification-node ${checksGate}`} onclick={() => void revealChecks()}><span class="mark" aria-hidden="true">{GATE_GLYPH[checksGate]}</span><span class="stage-text"><span class="map-label">Checks</span><strong>{candidatePassed ? "All checks passed" : "Not verified"}</strong><small>{candidateEvidence?.checks.length ?? 0} {(candidateEvidence?.checks.length ?? 0) === 1 ? "command" : "commands"} · View</small></span></button>
            <span class="wire" class:lit={checksGate === "ok"} aria-hidden="true"></span>
            <div class={`stage project-stage ${projectGate}`}>{#if projectGate === "wait"}<span class="mark" role="img" aria-label="writing"><span class="tui-spin"><span>|/-\</span></span></span>{:else}<span class="mark" aria-hidden="true">{GATE_GLYPH[projectGate]}</span>{/if}<span class="stage-text"><span class="map-label">project</span><strong>{projectGateText}</strong></span></div>
            <span class="wire last" class:lit={projectGate === "ok" || gateOpen} aria-hidden="true"></span>
            <div class="map-boundary" class:applied={presentation.state === "applied"}><span class="decision-aperture" aria-hidden="true"></span><details class="destination-node"><summary><span class="map-label">Applies to</span><strong>{run.repo_root.split(/[\\/]/).pop()}</strong><span>{presentation.state === "applied" ? "Applied · show folder" : "Show folder"}</span></summary><div class="node-pop"><code>{run.repo_root}</code><p>Apply writes these files to this project folder. Viewing it changes nothing.</p></div></details></div>
          </div>
        </div>
        <div class="map-caption"><button onclick={() => document.getElementById("review-apply-decision")?.scrollIntoView({ block: "nearest" })}>{presentation.state === "applied" ? "View outcome" : "Go to Apply"} ↓</button></div>
      </details>
    <div class="review-grid">
      <article class="panel files-panel">
        <div class="panel-title files-title">
          <div><p class="eyebrow">Exact files to Apply</p><h2>Prepared changes</h2></div>
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
              {@const diff = selectedDiff}
              <div class="comparison-context">
                <strong title={file.path}>{file.path}</strong>
                {#if cli}<span>by {cli}</span>{/if}
                {#if diff?.status === "ready"}<span class="diff-count"><b class="plus">+{diff.added}</b> <b class="minus">−{diff.removed}</b></span>{/if}
                <div class="comparison-mode" role="group" aria-label="Comparison view">
                  <button aria-pressed={comparisonMode === "changes"} onclick={() => chooseComparison("changes")}>Changes</button>
                  <button aria-pressed={comparisonMode === "exact"} onclick={() => chooseComparison("exact")}>Before &amp; after</button>
                </div>
              </div>
              {#if comparisonMode === "changes" && diff?.status === "ready"}
                <div class="line-diff" role="region" aria-label={`Changes in ${file.path}`}>
                  {#each diff.rows as row, index (`${file.path}:${index}`)}
                    {#if row.kind === "fold" && !openFolds.includes(`${file.path}:${index}`)}
                      <button class="diff-fold" onclick={() => (openFolds = [...openFolds, `${file.path}:${index}`])}>{row.count} unchanged lines</button>
                    {:else}
                      {#each row.kind === "fold" ? row.lines : [row] as line}
                        <div class={`diff-line ${line.kind}`}><span class="ln">{line.kind === "add" ? "" : line.before}</span><span class="ln">{line.kind === "del" ? "" : line.after}</span><span class="mark" aria-hidden="true">{line.kind === "add" ? "+" : line.kind === "del" ? "−" : ""}</span><code>{line.text}{#if line.ending !== "lf"}<span class="line-ending">{line.ending === "crlf" ? "CRLF" : "No newline at end of file"}</span>{/if}</code></div>
                      {/each}
                    {/if}
                  {:else}
                    <p class="diff-note">The file content is unchanged; only its mode changed.</p>
                  {/each}
                </div>
              {:else if comparisonMode === "changes" && diff && diff.status !== "binary" && diff.status !== "error"}
                <p class="diff-note" role="status">
                  {#if diff.status === "loading"}Loading changes…
                  {:else if diff.status === "too_large"}Too many changed lines to compare here. Showing the exact files instead.
                  {:else}This file is larger than the preview.<button onclick={() => loadWholeFile(file)}>Load the whole file to see changes</button>{/if}
                </p>
              {/if}
              {#if comparisonMode === "exact" || !diff || diff.status === "binary" || diff.status === "error" || diff.status === "too_large" || diff.status === "partial"}
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
                          <p class="binary-label">Binary file · first bytes in hex</p>
                          <pre class="binary-content">{hexContent(content)}{content.displayByteCount < content.loadedByteCount || !content.complete ? " …" : ""}</pre>
                          {#if content.displayByteCount < content.loadedByteCount}
                            <button class="expand-content" onclick={() => showAllLoadedBinary(file, side)}>
                              Show all loaded binary bytes
                            </button>
                          {/if}
                        {:else if content}
                          <pre class="text-content">{textContent(content)}{content.complete ? "" : "\n… exact content continues"}</pre>
                        {:else}
                          <p class="content-loading">Loading file…</p>
                        {/if}
                        {#if content && !content.complete}
                          <button class="expand-content" disabled={content.loading} onclick={() => loadPreparedContent(file, side, true)}>
                            {content.loading ? "Loading…" : "Load the whole file"}
                          </button>
                        {/if}
                      </section>
                    {/if}
                  {/each}
                </div>
              {/if}

              <details class="digest-details">
                <summary>File fingerprints</summary>
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
    <details class="technical-evidence" bind:this={evidenceDisclosure}><summary>Checks, plan &amp; technical details</summary><aside class="review-support" aria-label="Review evidence and ownership">
      <article class="panel evidence-panel">
        <div class="panel-title"><p class="eyebrow">Technical details</p><h2>Review evidence</h2></div>
        <dl class="evidence-list">
          <div><dt><IconShieldCheck size={14} /> Root</dt><dd>{run.repo_root}</dd></div>
          <div><dt>Profile</dt><dd>{review.enforcement.run.effective_profile}</dd></div>
          <div><dt>Paths</dt><dd>{manifest.summary.added + manifest.summary.modified + manifest.summary.deleted} · {manifest.summary.added} added · {manifest.summary.modified} modified · {manifest.summary.deleted} deleted</dd></div>
          <div><dt>State</dt><dd>{presentation.state}</dd></div>
          <div><dt>Checks on the combined changes</dt><dd class="candidate-checks" tabindex="-1">
            {#if candidatePassed && candidateEvidence}
              Passed · {candidateEvidence.checks.length} command{candidateEvidence.checks.length === 1 ? "" : "s"} run by Pytxo on these exact files. Checked {formatPreparedAt(candidateEvidence.verified_at)}.
              <ul>{#each candidateEvidence.checks as check}<li><code>{check.command}</code> · {check.task_id}</li>{/each}</ul>
              {#if candidateEvidence.exclusions.length}<span>Not part of the change set: {candidateEvidence.exclusions.join(", ")}.</span>{/if}
            {:else if candidateEvidence}
              Not verified. A check failed, did not finish, or is not supported on the combined changes.
            {:else}
              Not verified. Checks ran only in each agent's own copy, not on the combined changes.
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
        <div class="panel-title"><p class="eyebrow">Plan</p><h2>Who changed what</h2></div>
        <div class="task-list">
          {#each tasks as task (task.task_id)}
            <div>
              <span>Step {task.wave + 1}</span>
              <p><strong>{task.task_id}</strong><small>{recordedWorkerLabel(agents, run, task.task_id)} · {task.paths.join(", ")}</small></p>
              <em>{task.depends_on.length ? `after ${task.depends_on.join(", ")}` : "root task"}</em>
            </div>
          {/each}
        </div>
      </article>

      </aside>

      <article class="panel attempts-panel">
        <div class="panel-title"><p class="eyebrow">Apply</p><h2>Apply attempts</h2></div>
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
  <footer id="review-apply-decision" class:ready-decision={presentation.state === "ready" && !applyDisabledReason} class="decision-bar repository-boundary tui-pane" class:confirmed={presentation.state === "applied"} aria-label="Decision">
    <span class="tui-legend" aria-hidden="true"><b>apply</b>{#if manifest}<span class="gate-mini">{#each gateMarks as mark}<i class={mark}>{GATE_GLYPH[mark]}</i>{/each}</span>{/if}<span>→ {run.repo_root.split(/[\\/]/).pop()}</span></span>
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
          ? "Preparing review"
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
      <details class="decision-identity package-identity"><summary>Prepared change ID</summary><code>{review?.prepared_digest ?? "ID unavailable"}</code></details>
      <span class="decision-aperture" aria-hidden="true"></span>
      <div><strong>Applies to {run.repo_root.split(/[\\/]/).pop()}</strong><span>{presentation.state === "applied" ? "Applied" : presentation.state === "applying" ? "Applying · not finished" : "Not applied yet"}</span></div>
      <!-- A blocked reason equal to the decision copy is already shown above; show any other reason once, beside Apply. -->
      {#if applyDisabledReason && applyDisabledReason !== presentation.detail}
        <p class="boundary-reason blocked" id="apply-disabled-reason">{applyDisabledReason}</p>
      {:else}
        <p class="boundary-reason">{presentation.state === "applied" ? "As recorded when Apply finished." : "Apply writes exactly these files, nothing else."}</p>
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
        title={presentation.discardAllowed ? "Permanently remove the prepared changes" : presentation.detail}
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
        <h2 id="apply-title">Apply these {manifest?.files.length ?? 0} reviewed {manifest?.files.length === 1 ? "file" : "files"}?</h2>
        <p>
          Pytxo checks that your project still matches this review, then writes exactly
          these files to {run.repo_root.split(/[\\/]/).pop()}. Other files are left alone.
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
        <h2 id="discard-title">Discard these changes?</h2>
        <p>This permanently removes the prepared files and the agents' working copies. Your project is not changed.</p>
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
  .review-screen { padding-top:8px; gap:8px; }
  .review-decision { padding-bottom:0; }
  .review-header h1 { margin-block:4px; }
  .review-mode { display:inline; margin-left:12px; }
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
  .review-screen .files-title { padding:5px 12px; margin:0; }
  .review-screen .files-title h2 { font-size:14px; }
  .review-screen .review-grid { margin-top:0; }
  .review-header .back { min-height:20px; }
  .review-header h1 { font-size:22px; }
  @container run-review (max-width:899px) { .decision-bar.ready-decision>.decision-main:first-child { flex-wrap:wrap; } }

  /* Review reads as one code workspace, with the candidate map as its context. */
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
  .file-row.chosen .file-name small { color:var(--pytxo-text-body); }
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

  /* Review is a fixed Work state: evidence scrolls while the decision stays visible. */
  .review-screen{display:flex;width:100%;max-width:none;height:100%;min-height:0;box-sizing:border-box;flex-direction:column;overflow:hidden}
  .review-decision{flex:0 0 auto}
  .review-scroll{flex:1;min-height:0;overflow:auto;overscroll-behavior:contain;scrollbar-gutter:stable}
  .candidate-workspace{overflow:visible}
  .review-loading,.review-error,.review-unavailable{min-height:0;flex:1;overflow:auto}
  .decision-bar{position:relative;bottom:auto;flex:0 0 auto;margin:0;padding-block:8px;background:var(--pytxo-surface-shell)}
  @media(max-height:800px){.review-screen{gap:6px}.decision-bar{padding-block:2px}.diff-side .text-content{min-height:176px}}

  /* Line diff: computed in the renderer from the same exact bytes as "Before & after". */
  .comparison-context strong { min-width:0;overflow:hidden;color:var(--pytxo-text-strong);font:12px/1.4 "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap; }
  .diff-count { font:12px/1.4 "IBM Plex Mono",monospace; }
  .diff-count b { font-weight:500; }
  .diff-count .plus { color:var(--state-verified); }
  .diff-count .minus { color:var(--state-refuted); }
  .comparison-mode { display:flex;margin:-3px 0 -3px auto;padding:1px;border:1px solid var(--pytxo-line);border-radius:5px;background:var(--pytxo-surface-input); }
  .comparison-mode button { min-height:20px;padding:0 9px;border:0;border-radius:3px;background:transparent;color:var(--pytxo-text-muted);font:11px var(--pytxo-font-ui);cursor:pointer;transition:background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease),color var(--pytxo-motion-fast) var(--pytxo-motion-ease); }
  .comparison-mode button:hover { color:var(--pytxo-text-strong); }
  .comparison-mode button[aria-pressed="true"] { background:var(--pytxo-surface-active);color:var(--pytxo-text-strong); }
  .comparison-mode button:focus-visible { outline:2px solid var(--pytxo-accent);outline-offset:1px; }
  .line-diff { min-height:200px;max-height:52vh;overflow:auto;padding:6px 0;background:var(--pytxo-code-surface);font:12.5px/1.65 "IBM Plex Mono",monospace;tab-size:2; }
  .diff-line { display:grid;grid-template-columns:4ch 4ch 2ch minmax(0,1fr);gap:0 8px;padding:0 12px 0 8px; }
  .diff-line .ln { color:var(--pytxo-text-muted);opacity:.7;text-align:right;user-select:none;font-variant-numeric:tabular-nums; }
  .diff-line .mark { text-align:center;user-select:none; }
  .diff-line code { min-width:0;color:var(--pytxo-text-body);font:inherit;white-space:pre-wrap;overflow-wrap:anywhere; }
  .diff-line .line-ending { display:inline-block;margin-left:12px;color:var(--pytxo-text-muted);font:11px/1.6 var(--pytxo-font-ui, sans-serif);white-space:normal; }
  .diff-line.add { background:color-mix(in oklab,var(--state-verified) 13%,transparent); }
  .diff-line.add .mark,.diff-line.add .ln { color:var(--state-verified);opacity:1; }
  .diff-line.del { background:color-mix(in oklab,var(--state-refuted) 13%,transparent); }
  .diff-line.del .mark,.diff-line.del .ln { color:var(--state-refuted);opacity:1; }
  .diff-fold { display:block;width:100%;min-height:26px;margin:2px 0;padding:0 12px 0 calc(8ch + 24px);border:0;border-block:1px dashed var(--pytxo-line-soft);background:color-mix(in oklab,var(--pytxo-surface-raised) 60%,transparent);color:var(--pytxo-text-muted);font:11px var(--pytxo-font-ui);text-align:left;cursor:pointer; }
  .diff-fold:hover { color:var(--pytxo-text-strong);background:var(--pytxo-surface-hover); }
  .diff-fold:focus-visible { outline:2px solid var(--pytxo-accent);outline-offset:-2px; }
  .diff-note { display:flex;flex-wrap:wrap;align-items:center;gap:8px 12px;margin:0;padding:12px 14px;border-bottom:1px solid var(--pytxo-line-soft);color:var(--pytxo-text-soft);font-size:12px; }
  .diff-note button { min-height:28px;padding:0 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:transparent;color:var(--pytxo-text-strong);font:inherit;cursor:pointer; }
  @media (prefers-reduced-motion: reduce) { .comparison-mode button { transition:none; } }

  /* Apply gate: the changed files converge into one prepared change, which has
     to clear every condition before it can reach the project. Terminal language,
     same as the fleet: mono labels, one glyph per state, wires that carry signal
     only while Apply is actually available. */
  .candidate-overview { overflow:hidden; border:1px solid var(--pytxo-line); border-radius:6px; background:var(--pytxo-work-canvas); }
  .candidate-overview>summary { padding:7px 12px; gap:10px; background:color-mix(in srgb,var(--pytxo-surface-panel) 72%,transparent); font:12px var(--pytxo-font-mono); }
  .candidate-overview[open]>summary { border-bottom:1px solid var(--pytxo-line-soft); }
  .candidate-overview>summary strong { font:600 12px var(--pytxo-font-ui); }
  .candidate-symbol { color:var(--pytxo-activity); font:14px var(--pytxo-font-mono); }
  .gate-mini { display:inline-flex; gap:3px; margin-left:auto; font:600 12px var(--pytxo-font-mono); }
  .gate-mini i, .stage .mark { font-style:normal; color:var(--state-unknown); }
  .gate-mini i.ok, .stage.ok .mark { color:var(--state-verified); }
  .gate-mini i.fail, .stage.fail .mark { color:var(--state-refuted); }
  .gate-mini i.wait, .stage.wait .mark { color:var(--live); }
  .gate-mini + .verification-summary { margin-left:0; }
  .candidate-overview>summary::after { margin-left:14px; color:var(--pytxo-text-muted); }
  .decision-bar.tui-pane { --tui-bg:var(--pytxo-surface-shell); margin-top:12px; padding-top:14px; padding-bottom:8px; border-radius:6px; }
  .decision-bar .tui-legend .gate-mini { margin-left:0; }
  .decision-bar.confirmed { border-color:color-mix(in srgb,var(--state-verified) 45%,var(--pytxo-line)); }
  /* The code view grows into the space above the decision, like an editor, instead of leaving a gap. */
  .review-scroll { display:flex; flex-direction:column; }
  .review-scroll>.candidate-workspace, .speculative-workspace, .review-screen .review-grid, .review-grid>.files-panel, .file-content { display:flex; flex-direction:column; flex:1 0 auto; min-height:0; }
  .review-scroll .file-list { flex:1 0 auto; align-items:stretch; }
  .file-content>.line-diff, .file-content>.exact-diff { flex:1 0 auto; }
  .file-navigation { max-height:none; }
  .candidate-context { display:grid; grid-template-columns:minmax(120px,190px) 44px minmax(0,1fr); align-items:center; padding:10px 14px 4px; background:var(--pytxo-work-canvas); }
  .map-label { color:var(--pytxo-text-muted); font:500 10.5px var(--pytxo-font-mono); letter-spacing:.02em; white-space:nowrap; }
  .formation-files { display:grid; grid-template-columns:minmax(0,1fr) auto; gap:3px; min-width:0; }
  .formation-files .map-label { grid-row:1; }
  .formation-files button:not(.more-files) { grid-column:1 / -1; display:flex; justify-content:flex-start; gap:8px; min-height:24px; padding:2px 8px; border:1px solid var(--pytxo-line-soft); border-radius:3px; background:var(--pytxo-work-node); color:var(--pytxo-text-body); font:12px var(--pytxo-font-mono); text-align:left; }
  .formation-files button:not(.more-files):hover { border-color:color-mix(in srgb,var(--pytxo-activity) 46%,var(--pytxo-line)); }
  .formation-files button>span:last-child { overflow:hidden; text-overflow:ellipsis; white-space:nowrap; }
  .formation-files .map-selected { border-color:color-mix(in srgb,var(--pytxo-activity) 70%,var(--pytxo-line)); background:color-mix(in srgb,var(--pytxo-work-node) 82%,var(--pytxo-activity)); }
  .map-file-kind { font:600 10.5px var(--pytxo-font-mono); color:var(--state-attention); }
  .map-file-kind.add { color:var(--state-verified); } .map-file-kind.delete { color:var(--state-refuted); }
  .formation-files .more-files { grid-column:2; grid-row:1; min-height:18px; padding:0; border:0; background:transparent; color:var(--pytxo-text-soft); font:11px var(--pytxo-font-mono); }
  .convergence { width:100%; height:84px; fill:none; stroke:color-mix(in srgb,var(--pytxo-text-muted) 50%,var(--pytxo-line)); stroke-width:1.25; vector-effect:non-scaling-stroke; }
  .convergence path { vector-effect:non-scaling-stroke; }
  .convergence path.selected { stroke:var(--pytxo-activity); }
  .gate { display:grid; grid-template-columns:auto minmax(18px,1fr) auto minmax(18px,1fr) auto minmax(18px,1fr) auto minmax(18px,1fr) auto; align-items:center; min-width:0; }
  .gate .stage { display:flex; align-items:center; gap:9px; min-width:0; margin:0; padding:6px 8px; border:1px solid transparent; border-radius:4px; background:transparent; color:var(--pytxo-text-body); text-align:left; }
  .gate .stage .mark { display:grid; place-items:center; flex:none; width:26px; height:26px; border:1px solid currentColor; border-radius:3px; font:600 13px var(--pytxo-font-mono); }
  .gate .stage.open .mark { border-style:dashed; }
  .stage-text { display:grid; gap:2px; min-width:0; }
  .stage-text strong { font:500 12.5px var(--pytxo-font-ui); color:var(--pytxo-text-strong); white-space:nowrap; }
  .stage-text small { color:var(--pytxo-text-muted); font:11px var(--pytxo-font-mono); white-space:nowrap; overflow:hidden; text-overflow:ellipsis; }
  .gate .stage.fail .stage-text strong { color:var(--state-refuted); }
  .gate button.stage { min-height:0; justify-content:flex-start; font:inherit; }
  .gate button.stage:hover, .gate details.stage > summary:hover { background:var(--pytxo-surface-hover); }
  .gate details.stage { padding:0; }
  .gate details.stage > summary { display:flex; align-items:center; gap:9px; padding:6px 8px; border-radius:4px; cursor:pointer; list-style:none; }
  .gate details.stage > summary::-webkit-details-marker { display:none; }
  .gate details.stage[open] { position:relative; }
  .node-pop { position:absolute; top:calc(100% + 6px); left:0; z-index:4; display:grid; gap:6px; width:min(46ch,60vw); padding:10px 12px; border:1px solid var(--pytxo-line); border-radius:4px; background:var(--pytxo-surface-raised); box-shadow:0 12px 30px -18px rgba(0,0,0,.6); }
  .destination-node .node-pop { left:auto; right:0; }
  .node-pop code { color:var(--pytxo-text-body); font:11px/1.5 var(--pytxo-font-mono); overflow-wrap:anywhere; }
  .node-pop p { margin:0; color:var(--pytxo-text-soft); font-size:11px; line-height:1.5; }
  .project-stage .tui-spin { width:1ch; }
  /* A wire is a run of box-drawing dashes; lit wires carry a highlight toward the project. */
  .wire { position:relative; height:16px; min-width:0; overflow:hidden; color:var(--pytxo-line); font:12px/16px var(--pytxo-font-mono); white-space:nowrap; }
  .wire::before { content:"──────────────────────────────────────────────────────"; }
  .wire::after { content:"▸"; position:absolute; right:0; top:0; padding-left:2px; background:var(--pytxo-work-canvas); }
  .wire.lit { color:color-mix(in srgb,var(--state-verified) 62%,var(--pytxo-line)); }
  .gate-open .wire.lit::before { background:linear-gradient(90deg,transparent 0 40%,var(--state-verified) 50%,transparent 60% 100%) 0 0 / 300% 100%, color-mix(in srgb,var(--state-verified) 55%,var(--pytxo-line)); -webkit-background-clip:text; background-clip:text; color:transparent; animation:wire-flow 2.4s linear infinite; }
  .gate-open .wire:nth-of-type(2)::before { animation-delay:.3s; } .gate-open .wire:nth-of-type(3)::before { animation-delay:.6s; } .gate-open .wire:nth-of-type(4)::before { animation-delay:.9s; }
  @keyframes wire-flow { from { background-position:100% 0, 0 0; } to { background-position:0 0, 0 0; } }
  .map-boundary { display:flex; align-items:stretch; gap:10px; min-width:0; padding:4px 0 4px 6px; }
  .map-boundary .decision-aperture { height:auto; min-height:40px; }
  .destination-node { position:relative; min-width:0; }
  .destination-node summary { display:grid; gap:2px; cursor:pointer; list-style:none; }
  .destination-node summary::-webkit-details-marker { display:none; }
  .destination-node summary strong { font:500 12.5px var(--pytxo-font-ui); color:var(--pytxo-text-strong); }
  .destination-node summary>span:last-child { color:var(--pytxo-text-soft); font:11px var(--pytxo-font-mono); }
  .map-boundary.applied .destination-node summary>span:last-child { color:var(--state-verified); }
  .landed .wire::before { color:var(--state-verified); animation:wire-land .9s ease-out both; }
  .landed .map-boundary .decision-aperture { animation:aperture-land 1.2s ease-out; }
  @keyframes wire-land { from { clip-path:inset(0 100% 0 0); } to { clip-path:inset(0 0 0 0); } }
  @keyframes aperture-land { 0%,40% { box-shadow:none; } 60% { box-shadow:0 0 0 3px color-mix(in srgb,var(--state-verified) 40%,transparent), 0 0 18px var(--state-verified); } 100% { box-shadow:none; } }
  .map-caption { display:flex; justify-content:flex-end; padding:0 12px 4px; }
  .map-caption button { min-height:18px; padding:0; border:0; background:transparent; color:var(--pytxo-text-soft); font:11px var(--pytxo-font-mono); }
  .map-caption button:hover:not(:disabled) { background:transparent; color:var(--pytxo-text-strong); }
  .candidate-context :is(button,summary):focus-visible { outline:2px solid var(--pytxo-accent); outline-offset:2px; }
  @container run-review (max-width:1180px) {
    /* The file list sits right below; at this width the gate keeps the row to itself. */
    .candidate-context { grid-template-columns:minmax(0,1fr); padding-top:8px; }
    .formation-files, .convergence { display:none; }
  }
  @container run-review (max-width:760px) {
    .gate { grid-template-columns:minmax(0,1fr); }
    .wire { height:10px; margin-left:20px; width:1px; background:var(--pytxo-line); }
    .wire.lit { background:color-mix(in srgb,var(--state-verified) 62%,var(--pytxo-line)); }
    .wire::before, .wire::after { content:none; }
  }
  @media (max-height:800px) { .convergence { height:64px; } .gate .stage .mark { width:22px; height:22px; } }
  @media (prefers-reduced-motion: reduce) { .gate-open .wire.lit::before, .landed .wire::before, .landed .map-boundary .decision-aperture { animation:none; } }
  :global([data-force-reduced-motion]) .gate-open .wire.lit::before, :global([data-force-reduced-motion]) .landed .wire::before { animation:none; }
</style>
