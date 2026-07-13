<script lang="ts">
  import { markSetupComplete } from "../../lib/theme";
  import { Button } from "$lib/components/ui/button";

  let {
    onFinish,
    workspacePath = null as string | null,
  }: {
    onFinish: () => void;
    workspacePath?: string | null;
  } = $props();

  function enter() {
    markSetupComplete();
    onFinish();
  }

  async function requestNotify() {
    if (typeof Notification === "undefined") return;
    if (Notification.permission === "default") {
      try {
        await Notification.requestPermission();
      } catch {
        /* ignore */
      }
    }
  }
</script>

<div class="step">
  <h2 class="title">You're ready</h2>
  <p class="lead">
    Default trust is Orbit: agents write in a sandbox until you approve a flush (Blast Shield).
    Approvals show in the inspector when something needs a decision.
  </p>
  {#if workspacePath}
    <code class="path">{workspacePath}</code>
  {/if}
  <Button size="lg" onclick={enter}>Enter Pytxo Desktop</Button>
  <Button variant="ghost" size="sm" onclick={() => void requestNotify()}>
    Allow approval notifications
  </Button>
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
  .path {
    display: block;
    width: 100%;
    padding: 0.65rem 0.75rem;
    border-radius: var(--panel-radius);
    background: color-mix(in oklab, var(--card) 80%, transparent);
    font-size: 0.75rem;
    word-break: break-all;
    text-align: left;
    border: 1px solid var(--border);
  }
</style>
