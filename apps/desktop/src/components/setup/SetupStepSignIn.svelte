<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { ipc, onAuthChanged } from "../../lib/ipc";
  import { Button } from "$lib/components/ui/button";

  let {
    onContinue,
    onSkip,
  }: {
    onContinue: () => void;
    onSkip: () => void;
  } = $props();

  let signedIn = $state(false);
  let routingLinked = $state(false);
  let bridgeAvailable = $state(false);
  let routingCredentialPresent = $state(false);
  let routingRemoteStatus = $state<"absent" | "verified" | "unverified" | "revoked">("absent");
  let waiting = $state(false);
  let timedOut = $state(false);
  let polling: ReturnType<typeof setInterval> | null = null;
  let waitTimer: ReturnType<typeof setTimeout> | null = null;
  let unlisten: (() => void) | null = null;

  async function refresh(autoAdvance = false) {
    const [auth, routing] = await Promise.all([
      ipc.authStatus(),
      ipc.routingAccountStatus().catch(() => ({
        bridge_available: false,
        credential_present: false,
        account_id: null,
        expires_at: null,
        remote_status: "absent" as const,
      })),
    ]);
    signedIn = auth.signed_in;
    bridgeAvailable = routing.bridge_available;
    routingCredentialPresent = routing.credential_present;
    routingRemoteStatus = routing.remote_status;
    routingLinked = routing.remote_status === "verified";
    if (routingLinked || signedIn) {
      waiting = false;
      timedOut = false;
      if (autoAdvance) onContinue();
    }
  }

  async function signIn() {
    waiting = true;
    timedOut = false;
    if (waitTimer) clearTimeout(waitTimer);
    waitTimer = setTimeout(() => {
      if (!signedIn && !routingLinked) timedOut = true;
    }, 90_000);
    await ipc.authOpenSignIn();
  }

  async function resetAndSignIn() {
    await ipc.routingAccountDisconnect();
    await refresh();
    await signIn();
  }

  onMount(async () => {
    await refresh(true);
    unlisten = await onAuthChanged(() => refresh(true));
    polling = setInterval(() => refresh(false), 15_000);
  });

  onDestroy(() => {
    if (polling) clearInterval(polling);
    if (waitTimer) clearTimeout(waitTimer);
    unlisten?.();
  });
</script>

<div class="step">
  <h2 class="title">Sign in to Pytxo</h2>
  <p class="lead">
    Optional. Local Core runs work without an account. Browser sign-in connects
    experimental hosted Routing only; it does not change agent or Ultra billing.
  </p>

  {#if signedIn || routingLinked}
    <p class="ok">{routingLinked ? "Routing account linked" : "Existing account session present"}</p>
    <Button class="signin-cta" onclick={onContinue}>Continue</Button>
  {:else}
    {#if !bridgeAvailable}
      <p class="warn" role="status">Experimental Routing account connection is off. Continue with Local Core.</p>
    {:else if routingRemoteStatus === "revoked"}
      <p class="warn" role="status">This Routing connection was revoked. Connect again to use hosted Routing.</p>
      <Button class="signin-cta" onclick={resetAndSignIn}>Reconnect Routing</Button>
    {:else if routingCredentialPresent}
      <p class="warn" role="status">The stored Routing credential could not be verified. Check your connection and try again.</p>
      <Button class="signin-cta" onclick={() => refresh()}>Retry verification</Button>
    {:else}
    {#if waiting}
      <p class="waiting" role="status">Waiting for browser sign-in…</p>
    {/if}
    {#if timedOut}
      <p class="warn" role="alert">
        No account return yet. Finish in the browser, or skip and continue with Core.
      </p>
      <a class="link" href="https://pytxo.com/account" target="_blank" rel="noopener noreferrer">Open account help</a>
      <a class="link" href="https://discord.gg/AUFRPFjSYv" target="_blank" rel="noopener noreferrer">
        Discord
      </a>
    {/if}
    <Button class="signin-cta" onclick={signIn}>{waiting ? "Open sign-in again" : "Sign in with Pytxo"}</Button>
    {/if}
    <Button class="signin-cta" variant="ghost" onclick={onSkip}>Skip for now</Button>
  {/if}
</div>

<style>
  .step {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 1rem;
    max-width: 420px;
    margin: 0 auto;
  }
  .title {
    margin: 0;
    font-size: 1.35rem;
    text-wrap: balance;
  }
  .lead {
    margin: 0;
    color: var(--muted-foreground);
    font-size: 0.9rem;
    text-wrap: pretty;
    line-height: 1.5;
  }
  .ok {
    color: var(--primary);
    font-size: 0.9rem;
  }
  .waiting {
    margin: 0;
    font-size: 0.85rem;
    color: var(--primary);
  }
  .warn {
    margin: 0;
    font-size: 0.85rem;
    color: var(--muted-foreground);
  }
  .link {
    font-size: 0.85rem;
    color: var(--primary);
  }
  /* Named (not `.step button`) so this never leaks a min-width onto buttons
     rendered by other setup steps that also use the shared `.step` wrapper. */
  :global(.signin-cta) {
    min-width: 220px;
  }
</style>
