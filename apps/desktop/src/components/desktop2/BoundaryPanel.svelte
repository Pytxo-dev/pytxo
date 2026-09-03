<script lang="ts">
  import { IconAlertTriangle, IconArrowRight, IconLock } from "@tabler/icons-svelte";
  import {
    ATTEMPT_LABELS,
    SURFACE_LABELS,
    attemptTone,
    isPartiallyApplied,
    surfaceTone,
    worstTone,
    type EpistemicTone,
  } from "../../lib/epistemic";
  import type { HitlDto, PermissionEnforcementReceipt, RunDto, RunReviewDto } from "../../lib/types";
  import StateChip from "./StateChip.svelte";

  let {
    run,
    review = null,
    reviewError = null,
    loading = false,
    approvals = [],
    onOpenApprovals,
    onReview,
    onRecover = null,
  }: {
    run: RunDto | null;
    review?: RunReviewDto | null;
    reviewError?: string | null;
    loading?: boolean;
    approvals?: HitlDto[];
    onOpenApprovals: () => void;
    onReview: (runId: string) => void;
    onRecover?: ((runId: string) => void) | null;
  } = $props();

  type SurfaceKey = "workspace_isolation" | "host_filesystem_boundary" | "network" | "apply_boundary";

  const SURFACES: { key: SurfaceKey; label: string }[] = [
    { key: "workspace_isolation", label: "Workspace isolation" },
    { key: "host_filesystem_boundary", label: "Host filesystem" },
    { key: "network", label: "Network" },
    { key: "apply_boundary", label: "Apply boundary" },
  ];

  const receipt = $derived<PermissionEnforcementReceipt | null>(review?.enforcement?.run ?? null);

  /**
   * Every surface resolves to one of the four values the orchestrator reports.
   * With no receipt in hand the surfaces are `unknown` — not assumed safe, and
   * not shown as a failure either.
   */
  const rows = $derived(
    SURFACES.map(({ key, label }) => {
      const surface = receipt?.[key] ?? null;
      return {
        key,
        label,
        tone: surface ? surfaceTone(surface.status) : ("unknown" as EpistemicTone),
        status: surface ? SURFACE_LABELS[surface.status] : loading ? "Loading" : "Not reported",
        mechanism: surface?.mechanism ?? null,
        detail: surface?.detail ?? (reviewError ?? "No enforcement receipt in this snapshot"),
      };
    }),
  );

  /** A summary can never read stronger than its weakest surface. */
  const summaryTone = $derived(worstTone(rows.map((row) => row.tone), "unknown" as EpistemicTone));
  const summaryLabel = $derived(
    summaryTone === "verified"
      ? "All four surfaces enforced"
      : summaryTone === "claimed"
        ? "Partly advisory only"
        : summaryTone === "refuted"
          ? "A surface was bypassed"
          : "Enforcement not fully reported",
  );

  const attempts = $derived(review?.apply_attempts ?? []);
  const partial = $derived(attempts.find((attempt) => isPartiallyApplied(attempt)) ?? null);
  const pendingApproval = $derived(approvals[0] ?? null);
  const effectiveProfile = $derived((receipt?.effective_profile ?? run?.permission_profile ?? "").toLowerCase());
  const reviewAvailable = $derived(
    !!run && (
      !!run.prepared_digest ||
      !!review?.prepared_digest ||
      !!review?.prepared_manifest ||
      ["ready", "stale", "applied", "applying", "recovery_required", "discarded"].includes(
        (review?.apply_status ?? run.apply_status ?? "").toLowerCase(),
      )
    ),
  );

  const boundaryCopy = $derived(
    effectiveProfile === "orbit" || effectiveProfile === "galaxy"
      ? `${effectiveProfile === "orbit" ? "Orbit" : "Galaxy"} keeps prepared changes outside this repository until explicit Apply.`
      : effectiveProfile === "deepspace"
        ? "DeepSpace is non-flushable; it cannot Apply prepared changes to this repository."
        : effectiveProfile === "supernova"
          ? "Supernova writes directly. The reviewed Apply boundary does not contain repository changes."
          : "Repository behavior depends on the effective permission profile; check the enforcement receipt.",
  );

  function shortDigest(value: string | null | undefined) {
    if (!value) return "Not prepared";
    return value.length > 18 ? `${value.slice(0, 10)}…${value.slice(-6)}` : value;
  }

  function shortTime(value: string | null | undefined) {
    if (!value) return "Not recorded";
    const parsed = new Date(value);
    return Number.isNaN(parsed.getTime()) ? value : parsed.toLocaleString();
  }
</script>

<aside class="boundary" aria-labelledby="boundary-title">
  <header>
    <h2 id="boundary-title">Commit boundary</h2>
  </header>

  <section class="candidate">
    <p>Candidate outcome</p>
    {#if run}
      <dl>
        <div><dt>Run</dt><dd>{run.id}</dd></div>
        <div><dt>Base</dt><dd>{review?.base_revision ?? "Not reported"}</dd></div>
        <div><dt>Package</dt><dd>{shortDigest(run.prepared_digest)}</dd></div>
        <div><dt>Prepared</dt><dd>{shortTime(run.prepared_at)}</dd></div>
        <div><dt>Profile</dt><dd>{receipt?.effective_profile ?? run.permission_profile ?? "Not reported"}</dd></div>
      </dl>
    {:else}
      <div class="empty">No run is selected.</div>
    {/if}
  </section>

  <section class="enforcement" aria-label="Permission enforcement receipt">
    <div class="enforcement-head" data-tone={summaryTone}>
      <p>Enforcement receipt</p>
      <StateChip tone={summaryTone} label={summaryLabel} />
    </div>
    {#each rows as row (row.key)}
      <div class="surface" data-tone={row.tone}>
        <span class="surface-label">{row.label}</span>
        <StateChip tone={row.tone} label={row.status} />
        <span class="surface-detail" title={row.detail}>{row.mechanism ?? row.detail}</span>
      </div>
    {/each}
  </section>

  {#if pendingApproval}
    <button class="attention-row" data-tone="attention" onclick={onOpenApprovals}>
      <IconLock size={15} />
      <span><strong>A decision is waiting</strong><small>{pendingApproval.reason}</small></span>
      <IconArrowRight size={15} />
    </button>
  {/if}

  {#if partial}
    <!-- The one condition that must never be shown as a clean failure: the
         attempt stopped without a confirmed rollback, so the working tree may
         hold part of the change. -->
    <div class="partial" role="alert" data-tone="refuted">
      <IconAlertTriangle size={16} />
      <div>
        <strong>Working tree may be partially modified</strong>
        <small>
          Attempt {partial.attempt_id} ended as {ATTEMPT_LABELS[partial.outcome].toLowerCase()} and
          its rollback was not confirmed. Reconcile before starting another run.
        </small>
        {#if onRecover && run}
          <button onclick={() => onRecover(run.id)}>Reconcile recovery state</button>
        {/if}
      </div>
    </div>
  {/if}

  {#if attempts.length}
    <section class="attempts" aria-label="Apply attempts">
      <p>Apply attempts</p>
      {#each attempts as attempt (attempt.attempt_id)}
        {@const tone = attemptTone(attempt.outcome)}
        <div class="attempt" data-tone={tone}>
          <StateChip {tone} label={ATTEMPT_LABELS[attempt.outcome]} />
          <span class="attempt-meta">{shortTime(attempt.created_at)} · {attempt.phase}</span>
          {#if attempt.error_message}<small>{attempt.error_code}: {attempt.error_message}</small>{/if}
        </div>
      {/each}
    </section>
  {/if}

  <footer>
    <!-- One stable label. The most consequential control in the product does not
         change identity based on which state the run happens to be in. -->
    <button
      disabled={!reviewAvailable}
      title={reviewAvailable ? "Open the prepared package" : "Review becomes available after a package is prepared."}
      onclick={() => reviewAvailable && run && onReview(run.id)}
    >Review package<IconArrowRight size={15} /></button>
    <p><IconLock size={12} /> {boundaryCopy}</p>
  </footer>
</aside>

<style>
  .boundary{display:flex;min-width:0;flex-direction:column;overflow:hidden auto;border:1px solid var(--pytxo-line);border-radius:var(--pytxo-panel-radius,6px);background:var(--pytxo-surface-panel)}
  header{display:flex;align-items:center;gap:10px;min-height:40px;padding:8px 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  .candidate>p,.attempts>p,.enforcement-head p{margin:0 0 4px;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-transform:uppercase;letter-spacing:.06em}
  header h2{margin:0;font-size:14px;font-weight:600;letter-spacing:-.02em}

  .candidate{padding:12px 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  dl{display:flex;flex-direction:column;margin:9px 0 0;border:1px solid var(--pytxo-line-soft);border-radius:4px;background:var(--pytxo-surface-input)}
  dl>div{display:grid;grid-template-columns:66px minmax(0,1fr);gap:8px;padding:6px 9px;border-bottom:1px solid var(--pytxo-line-soft)}
  dl>div:last-child{border-bottom:0}
  dt{color:var(--pytxo-text-muted);font-size:11px}
  dd{overflow:hidden;margin:0;color:var(--pytxo-text-soft);font:11px "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap}
  .empty{padding:12px;border:1px solid var(--pytxo-line-soft);border-radius:4px;color:var(--pytxo-text-muted);font-size:11px}

  .enforcement{display:flex;flex-direction:column;padding:0;border-bottom:1px solid var(--pytxo-line-soft)}
  .enforcement-head{display:flex;align-items:center;justify-content:space-between;gap:10px;padding:11px 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  .surface{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:4px 10px;padding:9px 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  .surface:last-child{border-bottom:0}
  .surface-label{color:var(--pytxo-text-soft);font-size:12px;font-weight:550}
  .surface-detail{grid-column:1/-1;overflow:hidden;color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace;text-overflow:ellipsis;white-space:nowrap}

  .attention-row{display:flex;align-items:center;gap:9px;width:100%;padding:11px 14px;border:0;border-bottom:1px solid var(--pytxo-line-soft);border-left:2px solid var(--tone);background:color-mix(in oklab,var(--tone) 10%,transparent);color:inherit;cursor:pointer;font-family:inherit;text-align:left}
  .attention-row:hover{background:color-mix(in oklab,var(--tone) 16%,transparent)}
  .attention-row:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .attention-row span{display:flex;min-width:0;flex-direction:column;gap:2px}
  .attention-row strong{font-size:12px}
  .attention-row small,.partial small{overflow:hidden;color:var(--pytxo-text-muted);font-size:11px;line-height:1.4}
  .attention-row>:global(svg):first-child{color:var(--tone);flex:none}

  .partial{display:flex;align-items:flex-start;gap:9px;padding:11px 14px;border-bottom:1px solid var(--pytxo-line-soft);border-left:2px solid var(--tone);background:color-mix(in oklab,var(--tone) 10%,transparent)}
  .partial>:global(svg){color:var(--tone);flex:none}
  .partial div{display:flex;min-width:0;flex-direction:column;gap:3px}
  .partial strong{font-size:12px}
  .partial button{align-self:flex-start;margin-top:5px;height:28px;padding:0 9px;border:1px solid var(--tone);border-radius:4px;background:transparent;color:var(--tone);font-size:11px;cursor:pointer;font-family:inherit}

  .attempts{padding:12px 14px;border-bottom:1px solid var(--pytxo-line-soft)}
  .attempt{display:flex;flex-direction:column;gap:3px;padding:8px 0;border-bottom:1px solid var(--pytxo-line-soft)}
  .attempt:last-child{border-bottom:0;padding-bottom:0}
  .attempt-meta{color:var(--pytxo-text-muted);font:11px "IBM Plex Mono",monospace}
  .attempt small{color:var(--state-refuted);font-size:11px;line-height:1.4;overflow-wrap:anywhere}

  footer{padding:14px;margin-top:auto;border-top:1px solid var(--pytxo-line-soft)}
  footer button{display:flex;width:100%;height:38px;align-items:center;justify-content:center;gap:7px;border:1px solid transparent;border-radius:4px;background:var(--pytxo-text-strong);color:var(--pytxo-surface-shell);font-size:12px;font-weight:650;cursor:pointer}
  footer button:hover:not(:disabled){background:var(--pytxo-text-soft)}
  footer button:disabled{cursor:not-allowed;opacity:.38}
  footer button:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
  footer p{display:flex;align-items:flex-start;justify-content:center;gap:6px;margin:10px 4px 0;color:var(--pytxo-text-muted);font-size:11px;line-height:1.4;text-align:center}
</style>
