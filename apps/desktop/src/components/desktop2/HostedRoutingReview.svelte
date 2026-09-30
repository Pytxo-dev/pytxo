<script lang="ts">
  import { onMount } from "svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";
  import { ipc } from "../../lib/ipc";
  import type { FlowDraftRecord, ReviewedHostedAdvisorPacketPreview, RoutedAdvisorConsentStatus, RoutingHostedGrantStatus } from "../../lib/types";

  let { backend, history, domains, available }: {
    backend: DesktopBackend;
    history: FlowDraftRecord[];
    domains: { domain_id: string; repo_root: string }[];
    available: boolean;
  } = $props();
  let grants = $state<RoutingHostedGrantStatus[]>([]);
  let selected = $state<string | null>(null);
  let packet = $state<ReviewedHostedAdvisorPacketPreview | null>(null);
  let packetBody = $state("");
  let consent = $state<RoutedAdvisorConsentStatus | null>(null);
  let grant = $state<RoutingHostedGrantStatus | null>(null);
  let grantLoaded = $state(false);
  let loading = $state(false);
  let acting = $state(false);
  let error = $state("");
  let notice = $state("");
  let listError = $state("");
  let reconnectRequired = $state(false);
  let disposed = false;
  const reviews = $derived(history.filter((draft) => draft.status === "review_only" && !draft.dispatched_run_id).slice(0, 6));

  onMount(() => {
    void refreshGrants();
    return () => { disposed = true; };
  });

  function describe(cause: unknown) {
    if (cause && typeof cause === "object" && "message" in cause) return String(cause.message);
    return "Routing access changed or Link is unavailable. Inspect the current state and retry.";
  }

  function workspaceLabel(domainId: string) {
    return domains.find((domain) => domain.domain_id === domainId)?.repo_root ?? domainId;
  }

  async function refreshGrants() {
    try {
      const current = await backend.hostedGrants();
      const account = current.length ? await ipc.routingAccountStatus().catch(() => null) : null;
      if (!disposed) {
        grants = current;
        reconnectRequired = current.length > 0 && (!account?.account_id || account.remote_status === "revoked");
        listError = "";
      }
    } catch (cause) {
      if (!disposed) { grants = []; reconnectRequired = false; listError = describe(cause); }
    }
  }

  async function inspect(draft: FlowDraftRecord) {
    if (acting || loading) return;
    if (selected === draft.id) { selected = null; packet = null; return; }
    selected = draft.id;
    packet = null;
    consent = null;
    grant = null;
    grantLoaded = false;
    packetBody = "";
    error = "";
    notice = "";
    loading = true;
    try {
      const reviewed = await backend.previewReviewedHostedPacket(draft.id);
      if (reviewed.domain_id !== draft.domain_id) throw new Error("Reviewed workspace changed.");
      const body = new TextDecoder("utf-8", { fatal: true }).decode(Uint8Array.from(reviewed.packet_body));
      JSON.parse(body);
      const [local, remote] = await Promise.all([
        backend.hostedConsentStatus(reviewed.domain_id),
        backend.hostedGrantStatus(reviewed.domain_id),
      ]);
      if (selected !== draft.id || disposed) return;
      if (local.domain_id !== reviewed.domain_id || local.recipient_identity !== reviewed.recipient_identity) throw new Error("Reviewed recipient changed.");
      packet = reviewed;
      packetBody = body;
      consent = local;
      grant = remote;
      grantLoaded = true;
    } catch (cause) {
      if (selected === draft.id && !disposed) error = describe(cause);
    } finally {
      if (!disposed) loading = false;
    }
  }

  async function refreshSelected() {
    const current = packet;
    if (!current) return;
    const [local, remote] = await Promise.allSettled([
      backend.hostedConsentStatus(current.domain_id),
      backend.hostedGrantStatus(current.domain_id),
    ]);
    if (!disposed && packet === current) {
      consent = local.status === "fulfilled" ? local.value : null;
      grant = remote.status === "fulfilled" ? remote.value : null;
      grantLoaded = remote.status === "fulfilled";
      if (local.status === "rejected" || remote.status === "rejected") {
        error = "Current hosted access could not be fully inspected. The previous display was cleared; check the recorded grant list below.";
      }
    }
    await refreshGrants();
  }

  async function changeAccess(action: "allow_local" | "confirm_link" | "revoke_local" | "revoke_link") {
    const reviewed = packet;
    const local = consent;
    if (!reviewed || (action !== "revoke_link" && !local) || acting) return;
    acting = true;
    error = "";
    notice = "";
    try {
      if (action === "allow_local") {
        await backend.enableHostedConsent(selected!, reviewed.domain_id, reviewed.request_digest, reviewed.scope_digest, local!.revision);
        notice = "Local consent recorded. No packet or Jev request was sent.";
      } else if (action === "confirm_link") {
        await backend.enableHostedGrant(reviewed.domain_id);
        notice = "Link recorded the workspace grant. Routing remains review-only; no Jev request was sent.";
      } else if (action === "revoke_local") {
        await backend.revokeHostedConsent(reviewed.domain_id, local!.revision);
        notice = "Local hosted consent revoked.";
      } else {
        grant = await backend.revokeHostedGrant(reviewed.domain_id, local?.revision ?? 0);
        notice = "Link access revoked and Pytxo's local send gate closed. Inspect the original workspace Store before any new opt-in if it was unavailable.";
      }
    } catch (cause) {
      error = describe(cause);
    } finally {
      try { await refreshSelected(); } catch (cause) { consent = null; grant = null; grantLoaded = false; error = describe(cause); }
      acting = false;
    }
  }

  async function recoverGrant(binding: RoutingHostedGrantStatus) {
    if (acting) return;
    acting = true;
    listError = "";
    try {
      const local = await backend.hostedConsentStatus(binding.domain_id).catch(() => null);
      await backend.revokeHostedGrant(binding.domain_id, local?.revision ?? 0);
      notice = "Link access revoked and Pytxo's local send gate closed. Inspect the original workspace Store before any new opt-in if it was unavailable.";
    } catch (cause) {
      listError = describe(cause);
    } finally {
      await refreshGrants();
      try { await refreshSelected(); } catch { consent = null; grant = null; grantLoaded = false; }
      acting = false;
    }
  }

  async function reconnectForRevocation() {
    error = "";
    listError = "";
    notice = "";
    try {
      await ipc.routingAccountReconnectForRevocation();
      notice = "Sign in with the original Routing account, then retry revocation here. New grants remain disabled.";
    } catch (cause) {
      listError = describe(cause);
    }
  }
</script>

{#if available || reviews.length || grants.length || listError}
  <section class="hosted-review" aria-label="Experimental hosted Routing review">
    <h2>Hosted Routing review</h2>
    <p>Experimental packet and account-grant review only. Jev sending and live routing are disabled.</p>
    {#each reviews as draft (draft.id)}
      <div class="review-row">
        <strong>{draft.title}</strong>
        <button disabled={acting || loading} onclick={() => void inspect(draft)}>{selected === draft.id ? "Close packet" : "Inspect redacted packet"}</button>
      </div>
      {#if selected === draft.id}
        <div class="packet-details">
          {#if loading}<p role="status">Checking the saved review and original workspace Store…</p>{/if}
          {#if error}<p class="error" role="alert">{error}</p>{/if}
          {#if packet}
            <p>Recipient: <code>{packet.recipient_identity}</code></p>
            <p>Disclosure scope SHA-256: <code>{packet.scope_digest}</code></p>
            <p>Packet SHA-256: <code>{packet.packet_digest}</code></p>
            <pre aria-label="Exact reviewed hosted packet JSON">{packetBody}</pre>
            {#if !packet.recordable_shadow_context}<p>This review needs complete context and an explicitly declared task type before hosted advice can be recorded. Revoke any older access or create a new review.</p>{/if}
            <p>Local consent: {consent ? consent.enabled && consent.current_scope ? packet.recordable_shadow_context ? "allowed" : "stored, but ineligible for this review" : "off" : "unknown; original Store unavailable"}{consent ? ` · revision ${consent.revision}` : ""}</p>
            <p>Last recorded Link grant: {grantLoaded ? grant?.state ?? "none" : "unknown"}{grant?.remote_revision ? ` · revision ${grant.remote_revision}` : ""}. This is not a live send check.</p>
            {#if available && packet.recordable_shadow_context && grantLoaded && consent && !consent.enabled && consent.revision + 1 === packet.reviewed_consent_revision && (!grant || grant.state === "revoked")}
              <button disabled={acting} onclick={() => void changeAccess("allow_local")}>Allow this packet locally</button>
            {/if}
            {#if available && packet.recordable_shadow_context && grantLoaded && consent && consent.enabled && consent.current_scope && (!grant || grant.state === "grant_pending")}
              <button disabled={acting} onclick={() => void changeAccess("confirm_link")}>Confirm workspace grant with Link</button>
            {/if}
            {#if grant && grant.state !== "revoked"}
              <button disabled={acting} onclick={() => void changeAccess("revoke_link")}>{grant.state === "revoke_pending" ? "Retry Link revocation" : "Revoke hosted access"}</button>
            {:else if consent?.enabled}
              <button disabled={acting} onclick={() => void changeAccess("revoke_local")}>Revoke local hosted consent</button>
            {/if}
          {/if}
          {#if notice}<p role="status">{notice}</p>{/if}
        </div>
      {/if}
    {/each}
    {#if grants.length}
      <div class="recovery-list">
        <strong>Workspace grants needing attention</strong>
        {#if reconnectRequired}<p>The original Routing account needs a fresh connection before Link revocation can finish.</p><button disabled={acting} onclick={() => void reconnectForRevocation()}>Reconnect original account for revocation</button>{/if}
        {#each grants as binding (binding.domain_id)}
          <div class="review-row">
            <span>{workspaceLabel(binding.domain_id)} · last recorded {binding.state}</span>
            <button disabled={acting} onclick={() => void recoverGrant(binding)}>{binding.state === "revoke_pending" ? "Retry revocation" : "Revoke access"}</button>
          </div>
        {/each}
      </div>
    {/if}
    {#if listError}<p class="error" role="alert">{listError}</p>{/if}
    {#if notice && (!selected || !packet)}<p role="status">{notice}</p>{/if}
  </section>
{/if}

<style>
  .hosted-review{margin:16px 0;padding:16px;border:1px solid var(--pytxo-line);border-radius:5px;background:var(--pytxo-surface-raised)}
  h2{margin:0 0 6px;font-size:15px;color:var(--pytxo-text-strong)}
  p{margin:6px 0;color:var(--pytxo-text-soft);font-size:12px;line-height:1.5}
  .review-row{display:flex;align-items:center;justify-content:space-between;gap:12px;padding:9px 0;border-top:1px solid var(--pytxo-line);font-size:12px}
  .review-row strong,.recovery-list strong{color:var(--pytxo-text-strong)}
  .review-row span{overflow-wrap:anywhere}
  button{min-height:34px;padding:6px 10px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-input);color:var(--pytxo-text-strong);cursor:pointer;text-align:left}
  button:disabled{cursor:not-allowed;opacity:.55}
  button:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:2px}
  .packet-details{padding:8px 0 14px}
  .packet-details button{margin:6px 8px 0 0}
  pre{max-height:230px;overflow:auto;padding:12px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-input);color:var(--pytxo-text-strong);font:11px/1.5 "IBM Plex Mono",monospace;white-space:pre-wrap;overflow-wrap:anywhere}
  code{overflow-wrap:anywhere}
  .recovery-list{margin-top:14px}
  .error{color:var(--pytxo-danger,#f08080)}
  @media(max-width:640px){.review-row{align-items:flex-start;flex-direction:column}}
</style>
