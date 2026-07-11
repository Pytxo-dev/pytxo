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
  let waiting = $state(false);
  let timedOut = $state(false);
  let polling: ReturnType<typeof setInterval> | null = null;
  let waitTimer: ReturnType<typeof setTimeout> | null = null;
  let unlisten: (() => void) | null = null;

  async function refresh(autoAdvance = false) {
    const auth = await ipc.authStatus();
    signedIn = auth.signed_in;
    if (signedIn) {
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
      if (!signedIn) timedOut = true;
    }, 90_000);
    await ipc.authOpenSignIn();
  }

  onMount(async () => {
    await refresh(true);
    unlisten = await onAuthChanged(() => refresh(true));
    polling = setInterval(() => refresh(false), 2000);
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
    Optional. Local Core runs work without an account. Sign in for Ultra billing, cloud runs, and
    org policy. Completes in your browser, then returns here.
  </p>

  {#if signedIn}
    <p class="ok">Signed in</p>
    <Button onclick={onContinue}>Continue</Button>
  {:else}
    {#if waiting}
      <p class="waiting" role="status">Waiting for browser sign-in…</p>
    {/if}
    {#if timedOut}
      <p class="warn" role="alert">
        No callback yet. Finish sign-in in the browser, or skip and continue with Core.
      </p>
      <a class="link" href="https://pytxo.com/account" target="_blank" rel="noopener noreferrer">
        Open account help
      </a>
    {/if}
    <Button onclick={signIn}>{waiting ? "Open sign-in again" : "Sign in with Pytxo"}</Button>
    <Button variant="ghost" onclick={onSkip}>Skip for now</Button>
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
  :global(.step button) {
    min-width: 220px;
  }
</style>
