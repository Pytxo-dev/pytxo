<script lang="ts">
  import SetupStepWelcome from "./SetupStepWelcome.svelte";
  import SetupStepCli from "./SetupStepCli.svelte";
  import SetupStepSignIn from "./SetupStepSignIn.svelte";
  import SetupStepWorkspace from "./SetupStepWorkspace.svelte";
  import SetupStepDone from "./SetupStepDone.svelte";

  type Step = "welcome" | "cli" | "signin" | "workspace" | "done";

  let {
    onComplete,
    onWorkspaceSelected,
  }: {
    onComplete: () => void;
    onWorkspaceSelected: (path: string) => void;
  } = $props();

  let step = $state<Step>("welcome");

  function next(s: Step) {
    step = s;
  }
</script>

<div class="setup">
  <div class="setup__card glass-panel">
    <header class="setup__header">
      <img src="/logo.png" alt="" width="32" height="32" class="setup__logo" />
      <span class="setup__brand chroma-text">Pytxo Reality Deck</span>
      <nav class="setup__steps" aria-label="Setup progress">
        {#each ["welcome", "cli", "signin", "workspace", "done"] as s, i}
          <span class="dot" class:active={step === s} class:done={["welcome", "cli", "signin", "workspace", "done"].indexOf(step) > i}></span>
        {/each}
      </nav>
    </header>

    <div class="setup__body">
      {#if step === "welcome"}
        <SetupStepWelcome onContinue={() => next("cli")} />
      {:else if step === "cli"}
        <SetupStepCli onContinue={() => next("signin")} onSkip={() => next("signin")} />
      {:else if step === "signin"}
        <SetupStepSignIn onContinue={() => next("workspace")} onSkip={() => next("workspace")} />
      {:else if step === "workspace"}
        <SetupStepWorkspace
          onContinue={() => next("done")}
          onSkip={() => next("done")}
          {onWorkspaceSelected}
        />
      {:else}
        <SetupStepDone onFinish={onComplete} />
      {/if}
    </div>
  </div>
</div>

<style>
  .setup {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem 1rem;
    min-height: 0;
  }
  .setup__card {
    width: min(520px, 100%);
    padding: 1.5rem;
    border-radius: 20px;
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.35), 0 0 0 1px color-mix(in oklab, var(--foreground) 6%, transparent);
  }
  .setup__header {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    margin-bottom: 1.75rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid color-mix(in oklab, var(--foreground) 8%, transparent);
  }
  .setup__logo {
    border-radius: 8px;
    outline: 1px solid rgba(255, 255, 255, 0.08);
  }
  .setup__brand {
    font-weight: 600;
    font-size: 0.95rem;
    flex: 1;
  }
  .setup__steps {
    display: flex;
    gap: 0.35rem;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: color-mix(in oklab, var(--foreground) 20%, transparent);
  }
  .dot.active {
    background: var(--brand-teal);
    box-shadow: 0 0 8px color-mix(in oklab, var(--brand-teal) 50%, transparent);
  }
  .dot.done {
    background: color-mix(in oklab, var(--brand-teal) 60%, transparent);
  }
  .setup__body {
    min-height: 280px;
    display: flex;
    align-items: center;
  }
</style>
