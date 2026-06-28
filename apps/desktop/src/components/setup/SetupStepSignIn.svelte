<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { ipc, onAuthChanged } from "../../lib/ipc";

  let {
    onContinue,
    onSkip,
  }: {
    onContinue: () => void;
    onSkip: () => void;
  } = $props();

  let signedIn = $state(false);
  let polling: ReturnType<typeof setInterval> | null = null;
  let unlisten: (() => void) | null = null;

  async function refresh() {
    const auth = await ipc.authStatus();
    signedIn = auth.signed_in;
    if (signedIn) {
      onContinue();
    }
  }

  async function signIn() {
    await ipc.authOpenSignIn();
  }

  onMount(async () => {
    await refresh();
    unlisten = await onAuthChanged(refresh);
    polling = setInterval(refresh, 2000);
  });

  onDestroy(() => {
    if (polling) clearInterval(polling);
    unlisten?.();
  });
</script>

<div class="step">
  <h2 class="title">Sign in to Pytxo</h2>
  <p class="lead">
    Connect your account for Ultra billing, cloud runs, and org policy. Opens in your browser —
    you'll return here automatically.
  </p>

  {#if signedIn}
    <p class="ok">Signed in</p>
    <button type="button" class="primary" onclick={onContinue}>Continue</button>
  {:else}
    <button type="button" class="primary" onclick={signIn}>Sign in with Pytxo</button>
    <button type="button" class="ghost" onclick={onSkip}>Skip for now</button>
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
    color: var(--brand-teal);
    font-size: 0.9rem;
  }
  button {
    min-height: 44px;
    min-width: 220px;
    border-radius: 10px;
  }
  .ghost {
    background: transparent;
    border: none;
    color: var(--muted-foreground);
  }
</style>
