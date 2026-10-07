<script lang="ts">
  import FleetBoard from "../../src/components/desktop2/FleetBoard.svelte";
  import type { DesktopBackend } from "../../src/lib/desktop-backend";
  import type { AgentDto, RunDto, RunReviewDto } from "../../src/lib/types";

  let { backend, checks = ["npm test"] }: { backend: DesktopBackend; checks?: string[] } = $props();
  let runId = $state("run-a");
  let settled = $state(false);
  const run = $derived({ id: runId, domain_id: "domain", started_at: new Date(0).toISOString() } as RunDto);
  const agents = $derived(["one", "two"].map(id => ({
    id, task_id: id, run_id: runId, domain_id: "domain", wave: 0, root_id: null,
    status: settled ? "completed" : "running", exit_code: settled ? 0 : null,
    launcher: { id: "codex", display_name: id },
  } as AgentDto)));
  const review = $derived({ plan: { waves: [["one", "two"].map(task_id => ({
    task_id, agent: "codex", paths: [], depends_on: [], wave: 0, root: null, verify: checks,
  }))] }, prepared_manifest: null } as RunReviewDto);

  export function switchRun(id: string) { runId = id; }
  export function complete() { settled = true; }
</script>

<FleetBoard {run} {review} {agents} {backend} onInspect={() => {}} />
