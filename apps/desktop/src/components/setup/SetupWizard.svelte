<script lang="ts">
  import SetupShell from "./SetupShell.svelte";
  import SetupStepWelcome from "./SetupStepWelcome.svelte";
  import SetupStepCli from "./SetupStepCli.svelte";
  import SetupStepSignIn from "./SetupStepSignIn.svelte";
  import SetupStepWorkspace from "./SetupStepWorkspace.svelte";
  import SetupStepDone from "./SetupStepDone.svelte";
  import { Button } from "$lib/components/ui/button";

  type Step = "welcome" | "cli" | "signin" | "workspace" | "done";

  const STEPS: Step[] = ["welcome", "cli", "signin", "workspace", "done"];
  const STEP_LABELS: Record<Step, string> = {
    welcome: "Welcome",
    cli: "CLI",
    signin: "Account",
    workspace: "Workspace",
    done: "Ready",
  };

  let {
    onComplete,
    onWorkspaceSelected,
  }: {
    onComplete: () => void;
    onWorkspaceSelected: (path: string) => Promise<void> | void;
  } = $props();

  let step = $state<Step>("welcome");
  let workspacePath = $state<string | null>(null);
  let workspaceError = $state("");

  function next(s: Step) {
    step = s;
  }

  function back() {
    const i = STEPS.indexOf(step);
    if (i > 0) step = STEPS[i - 1]!;
  }

  async function handleWorkspaceSelected(path: string) {
    workspaceError = "";
    try {
      await onWorkspaceSelected(path);
      workspacePath = path;
    } catch (e) {
      workspaceError = e instanceof Error ? e.message : String(e);
      throw e;
    }
  }

  const stepIndex = $derived(STEPS.indexOf(step));
</script>

<SetupShell>
  <header class="setup__header">
    <img src="/logo.png" alt="" width="28" height="28" class="setup__logo" />
    <span class="setup__brand">Pytxo Desktop</span>
    <nav class="setup__progress" aria-label="Setup progress">
      {#each STEPS as s, i}
        <span
          class="setup__step"
          class:setup__step--active={step === s}
          class:setup__step--done={stepIndex > i}
          title={STEP_LABELS[s]}
        >
          <span class="setup__segment"></span>
          <span class="setup__label">{STEP_LABELS[s]}</span>
        </span>
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
        onWorkspaceSelected={handleWorkspaceSelected}
        error={workspaceError}
      />
    {:else}
      <SetupStepDone workspacePath={workspacePath} onFinish={onComplete} />
    {/if}
  </div>

  {#if step !== "welcome"}
    <footer class="setup__footer">
      <Button variant="ghost" size="sm" onclick={back}>Back</Button>
    </footer>
  {/if}
</SetupShell>

<style>
  .setup__header {
    display: flex;
    align-items: center;
    gap: 0.65rem;
    margin-bottom: 1.75rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid var(--border);
    flex-wrap: wrap;
  }
  .setup__logo {
    border-radius: var(--panel-radius);
  }
  .setup__brand {
    font-weight: 600;
    font-size: 0.95rem;
    flex: 1;
    color: var(--foreground);
    min-width: 6rem;
  }
  .setup__progress {
    display: flex;
    gap: 0.5rem;
    align-items: flex-end;
  }
  .setup__step {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    align-items: center;
    min-width: 3.25rem;
  }
  .setup__segment {
    width: 100%;
    height: 3px;
    border-radius: 2px;
    background: var(--border);
  }
  .setup__step--active .setup__segment {
    background: var(--primary);
  }
  .setup__step--done .setup__segment {
    background: color-mix(in oklab, var(--primary) 55%, var(--border));
  }
  .setup__label {
    font-size: 0.65rem;
    color: var(--muted-foreground);
    letter-spacing: 0.02em;
  }
  .setup__step--active .setup__label {
    color: var(--foreground);
  }
  .setup__body {
    min-height: 280px;
    display: flex;
    align-items: center;
  }
  .setup__footer {
    margin-top: 1rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border);
  }
</style>
