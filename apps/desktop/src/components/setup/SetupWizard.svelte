<script lang="ts">
  import SetupShell from "./SetupShell.svelte";
  import SetupStepWelcome from "./SetupStepWelcome.svelte";
  import SetupStepCli from "./SetupStepCli.svelte";
  import SetupStepAgents from "./SetupStepAgents.svelte";
  import SetupStepWorkspace from "./SetupStepWorkspace.svelte";
  import { markSetupComplete } from "../../lib/theme";
  import { setContext } from "svelte";

  // Three screens; terminal tools stay one optional detour from Agents.
  // Text size and theme live in Settings, with sensible defaults.
  type Step = "welcome" | "cli" | "agents" | "workspace";
  const STEPS: Step[] = ["welcome", "agents", "workspace"];
  const STEP_LABELS: Record<Step, string> = { welcome: "Welcome", cli: "Agents", agents: "Agents", workspace: "Project" };

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

  function back() {
    if (step === "cli") { step = "agents"; return; }
    const i = STEPS.indexOf(step);
    if (i > 0) step = STEPS[i - 1]!;
  }

  function finish() {
    markSetupComplete();
    onComplete(workspacePath !== null);
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

  const activeStage = $derived(step === "cli" ? "agents" : step);
  const index = $derived(STEPS.indexOf(activeStage));
</script>

<SetupShell>
  <header class="setup-bar">
    <strong>pytxo</strong>
    <nav class="setup-progress" aria-label="Setup progress">
      {#each STEPS as s, i}<span class:on={i <= index} aria-current={activeStage === s ? "step" : undefined} ><b class="sr">{STEP_LABELS[s]}{i < index ? ", done" : ""}</b></span>{/each}
    </nav>
    <span class="setup-count" aria-hidden="true">{String(index + 1).padStart(2, "0")} / {String(STEPS.length).padStart(2, "0")}</span>
  </header>

  <main class="setup-body">
    {#key step}
      <div class="setup-enter">
        {#if step === "welcome"}
          <SetupStepWelcome onContinue={() => step = "agents"} />
        {:else if step === "cli"}
          <SetupStepCli onContinue={() => step = "agents"} onSkip={() => step = "agents"} />
        {:else if step === "agents"}
          <SetupStepAgents onRuntimeTools={() => step = "cli"} onContinue={() => step = "workspace"} onSkip={() => step = "workspace"} />
        {:else}
          <SetupStepWorkspace bind:selected={workspacePath} onContinue={finish} onSkip={finish} onWorkspaceSelected={handleWorkspaceSelected} error={workspaceError} />
        {/if}
      </div>
    {/key}
  </main>

  <footer class="setup-foot" aria-hidden="true">Describe <b>→</b> Work <b>→</b> Review <b>→</b> Apply</footer>
</SetupShell>

<style>
  .setup-bar { position: relative; display: grid; grid-template-columns: 1fr auto 1fr; align-items: center; height: 56px; padding: 0 28px; }
  .setup-bar strong { font-size: 15px; font-weight: 600; letter-spacing: -0.01em; }
  .setup-progress { display: flex; gap: 6px; }
  .setup-progress span { width: 28px; height: 2px; border-radius: 2px; background: color-mix(in oklab, var(--foreground) 16%, transparent); transition: background-color 240ms ease; }
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  .setup-progress span.on { background: var(--foreground); }
  .setup-count { justify-self: end; color: var(--muted-foreground); font: 500 12px var(--font-mono, "IBM Plex Mono", monospace); letter-spacing: .06em; }
  .setup-body { position: relative; display: flex; min-height: 0; overflow: hidden; padding: 0 24px; }
  .setup-enter { display: flex; flex: 1; min-width: 0; min-height: 0; animation: setup-enter 260ms cubic-bezier(.2, .8, .2, 1); }
  .setup-foot { position: relative; padding: 18px; color: var(--muted-foreground); font: 500 11px var(--font-mono, "IBM Plex Mono", monospace); letter-spacing: .08em; text-align: center; text-transform: uppercase; }
  .setup-foot b { color: var(--foreground); font-weight: 500; opacity: .6; }
  @keyframes setup-enter { from { opacity: 0; transform: translateY(6px); } }
  @media (prefers-reduced-motion: reduce) { .setup-enter { animation: none; } .setup-progress span { transition: none; } }
  :global([data-force-reduced-motion]) .setup-enter { animation: none; }
  @media (max-height: 640px) { .setup-foot { display: none; } .setup-bar { height: 44px; } }
</style>
