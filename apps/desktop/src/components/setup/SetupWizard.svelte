<script lang="ts">
  import SetupShell from "./SetupShell.svelte";
  import SetupStepWelcome from "./SetupStepWelcome.svelte";
  import SetupStepCli from "./SetupStepCli.svelte";
  import SetupStepAgents from "./SetupStepAgents.svelte";
  import SetupStepWorkspace from "./SetupStepWorkspace.svelte";
  import SetupStepDisplay from "./SetupStepDisplay.svelte";
  import SetupStepDone from "./SetupStepDone.svelte";
  import { setContext } from "svelte";

  type Step = "welcome" | "cli" | "agents" | "workspace" | "display" | "done";

  const STEPS: Step[] = ["welcome", "agents", "workspace", "done"];
  const STEP_LABELS: Record<Step, string> = {
    welcome: "Welcome",
    cli: "CLI",
    agents: "Agent",
    workspace: "Project",
    display: "Display",
    done: "Ready",
  };

  let {
    onComplete,
    onWorkspaceSelected,
  }: {
    onComplete: (openGuidedFlow?: boolean) => void;
    onWorkspaceSelected: (path: string) => Promise<void> | void;
  } = $props();

  let step = $state<Step>("welcome");
  let workspacePath = $state<string | null>(null);
  let workspaceError = $state("");

  function next(s: Step) {
    step = s;
  }

  function back() {
    if (step === "cli") { step = "agents"; return; }
    if (step === "display") { step = "done"; return; }
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

  setContext("setup-navigation", { get canGoBack() { return step !== "welcome"; }, back });

  const activeStage = $derived(step === "cli" ? "agents" : step === "display" ? "done" : step);
</script>

<SetupShell height={step === "agents" || step === "workspace" ? 640 : step === "welcome" ? 570 : 500}>
  <aside class="setup__navigation">
    <nav class="setup__progress" aria-label="Setup progress">
      {#each STEPS as s, i}
        <span class="setup__step" class:active={activeStage === s} aria-current={activeStage === s ? "step" : undefined}>
          <span class="setup__number">{i + 1}</span>{STEP_LABELS[s]}
        </span>
      {/each}
    </nav>
  </aside>

  <div class="setup__body">
    {#if step === "welcome"}
      <SetupStepWelcome onContinue={() => next("agents")} />
    {:else if step === "cli"}
      <SetupStepCli onContinue={() => next("agents")} onSkip={() => next("agents")} />
    {:else if step === "agents"}
      <SetupStepAgents onRuntimeTools={() => next("cli")} onContinue={() => next("workspace")} onSkip={() => next("workspace")} />
    {:else if step === "workspace"}
      <SetupStepWorkspace
        bind:selected={workspacePath}
        onContinue={() => next("done")}
        onSkip={() => next("done")}
        onWorkspaceSelected={handleWorkspaceSelected}
        error={workspaceError}
      />
    {:else if step === "display"}
      <SetupStepDisplay onContinue={() => next("done")} />
    {:else}
      <SetupStepDone
        onDisplay={() => next("display")}
        workspacePath={workspacePath}
        onFinish={() => onComplete(workspacePath !== null)}
      />
    {/if}
  </div>

</SetupShell>

<style>
  .setup__navigation { padding: 16px 28px 0; border-bottom: 1px solid var(--border); }
  .setup__progress { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 12px; }
  .setup__step { display: flex; align-items: center; gap: 8px; min-height: 44px; padding-bottom: 12px; font-size: 13px; color: var(--muted-foreground); border-bottom: 2px solid transparent; }
  .setup__step.active { border-bottom-color: var(--foreground); color: var(--foreground); font-weight: 600; }
  .setup__number { font-size: 11px; font-variant-numeric: tabular-nums; opacity: .7; }
  .setup__body { min-width: 0; min-height: 0; padding: 24px 28px; }
  @media (max-width: 700px), (max-height: 650px) {
    .setup__navigation { padding: 8px 16px 0; }
    .setup__body { padding: 16px; }
    .setup__progress { gap: 8px; }
    .setup__step { gap: 6px; font-size: 12px; }
  }
</style>
