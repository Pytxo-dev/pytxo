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
  const STEP_HINTS: Record<Step, string> = {
    welcome: "How Pytxo works",
    cli: "Optional terminal tools",
    agents: "Connect your coding CLIs",
    workspace: "Choose a folder or the example",
    display: "Text size and theme",
    done: "Start your first job",
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

<SetupShell>
  <aside class="setup__navigation">
    <div class="setup__brand"><strong>Pytxo</strong><span>Desktop · beta</span></div>
    <nav class="setup__progress" aria-label="Setup progress">
      {#each STEPS as s, i}
        {@const done = i < STEPS.indexOf(activeStage)}
        <span class="setup__step" class:active={activeStage === s} class:done aria-current={activeStage === s ? "step" : undefined}>
          <span class="setup__number" aria-hidden={done ? "true" : undefined}>{#if done}<svg viewBox="0 0 16 16" width="12" height="12"><path d="M3 8.5 6.5 12 13 4.5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>{:else}{i + 1}{/if}</span>
          <span class="setup__label">{STEP_LABELS[s]}<small>{s === "workspace" && workspacePath ? workspacePath.split(/[\\/]/).pop() : STEP_HINTS[s]}</small></span>
        </span>
      {/each}
    </nav>
    <p class="setup__assurance"><span>Describe → Agents work → Review → Apply</span>Your project changes only when you Apply.</p>
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
  .setup__navigation { display: flex; flex-direction: column; gap: 26px; min-height: 0; padding: 28px 22px 24px; border-right: 1px solid var(--border); background: color-mix(in oklab, var(--card) 55%, var(--background)); }
  .setup__brand { display: grid; gap: 3px; padding: 0 8px; }
  .setup__brand strong { position: relative; width: max-content; font-size: 17px; font-weight: 650; letter-spacing: -0.02em; }
  .setup__brand strong::after { content: ""; position: absolute; left: 0; right: 0; bottom: -6px; height: 2px; border-radius: 2px; background: var(--pytxo-aperture-horizontal); }
  .setup__brand span { margin-top: 6px; font-size: 12px; color: var(--muted-foreground); }
  .setup__progress { display: grid; gap: 4px; }
  .setup__step { position: relative; display: grid; grid-template-columns: 26px minmax(0, 1fr); align-items: center; gap: 12px; min-height: 52px; padding: 7px 10px; border-radius: 8px; color: var(--muted-foreground); transition: background-color 160ms ease, color 160ms ease; }
  .setup__step.active { background: color-mix(in oklab, var(--foreground) 7%, transparent); color: var(--foreground); }
  .setup__step.active::before { content: ""; position: absolute; left: -22px; top: 10px; bottom: 10px; width: 2px; border-radius: 2px; background: var(--pytxo-aperture); }
  .setup__step.done { color: var(--foreground); }
  .setup__number { display: grid; place-items: center; width: 26px; height: 26px; border-radius: 50%; border: 1px solid var(--border); font-size: 11px; font-variant-numeric: tabular-nums; }
  .setup__step.active .setup__number { border-color: var(--foreground); background: var(--foreground); color: var(--background); font-weight: 650; }
  .setup__step.done .setup__number { border-color: color-mix(in oklab, var(--state-verified) 60%, var(--border)); color: var(--state-verified); }
  .setup__label { display: grid; gap: 2px; min-width: 0; font-size: 13px; font-weight: 600; }
  .setup__label small { overflow: hidden; font-size: 12px; font-weight: 400; color: var(--muted-foreground); text-overflow: ellipsis; white-space: nowrap; }
  .setup__assurance { display: grid; gap: 6px; margin: auto 0 0; padding: 14px 8px 0; border-top: 1px solid var(--border); font-size: 12px; line-height: 1.45; color: var(--muted-foreground); }
  .setup__assurance span { font: 11px var(--font-mono, ui-monospace, monospace); color: var(--foreground); opacity: .8; }
  .setup__body { min-width: 0; min-height: 0; padding: 34px 40px 26px; }
  @media (prefers-reduced-motion: reduce) { .setup__step { transition: none; } }
  /* Small windows: the rail folds into a row of tabs above the step. */
  @media (max-width: 860px), (max-height: 600px) {
    .setup__navigation { gap: 0; padding: 8px 16px 0; border-right: 0; border-bottom: 1px solid var(--border); background: transparent; }
    .setup__brand, .setup__assurance, .setup__label small { display: none; }
    .setup__progress { grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; }
    .setup__step { grid-template-columns: auto minmax(0, 1fr); gap: 6px; min-height: 40px; padding: 0 0 10px; border-radius: 0; border-bottom: 2px solid transparent; background: transparent; font-size: 12px; }
    .setup__step.active { border-bottom-color: var(--foreground); background: transparent; }
    .setup__step.active::before { display: none; }
    .setup__number { width: auto; height: auto; border: 0; opacity: .7; }
    .setup__step.active .setup__number { background: transparent; color: inherit; }
    .setup__label { font-size: 12px; }
    .setup__body { padding: 16px; }
  }
</style>
