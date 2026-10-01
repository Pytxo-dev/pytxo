import type { AgentDto, RoutingDisplaySummary, RunDto, RunReviewDto } from "./types";

/** Both the canvas and Work activity must reject missing or ambiguous rows. */
export function exactAttemptAgent(
  agents: AgentDto[],
  run: Pick<RunDto, "id" | "domain_id">,
  taskId: string,
  agentId: string,
): AgentDto | undefined {
  const matches = agents.filter(agent => agent.id === agentId && agent.task_id === taskId
    && agent.run_id === run.id && agent.domain_id === run.domain_id);
  return matches.length === 1 ? matches[0] : undefined;
}

/** Geometry reflects recorded dependency order, never runtime progress. */
export function executionTopology(
  plan: RunReviewDto["plan"] | null,
  agents: AgentDto[],
  run: Pick<RunDto, "id" | "domain_id"> & Partial<Pick<RunDto, "routing_revision">>,
  orientation: "horizontal" | "vertical" = "horizontal",
  routingSummary: RoutingDisplaySummary | null = null,
  routingIdentityUnknown = false,
) {
  const routed = run.routing_revision != null || routingIdentityUnknown;
  const matchingRouting = routed
    && routingSummary?.domain_id === run.domain_id
    && routingSummary.run_id === run.id
    && routingSummary.routing_revision === run.routing_revision
      ? routingSummary : null;
  const waves = plan?.waves.filter(wave => wave.length) ?? [];
  const rowCount = Math.max(1, ...waves.map(wave => wave.length));
  // Scene bounds include the rendered 224 x 100 worker node. Fit and the
  // minimap can therefore reason about exact content bounds instead of node
  // centres, which previously clipped the first and last waves.
  const horizontalInset = 136;
  const verticalInset = 64;
  const width = orientation === "vertical"
    ? Math.max(320, horizontalInset * 2 + Math.max(0, rowCount - 1) * 264)
    : Math.max(640, horizontalInset * 2 + Math.max(0, waves.length - 1) * 320);
  const height = orientation === "vertical"
    ? Math.max(360, verticalInset * 2 + Math.max(0, waves.length - 1) * 154)
    : Math.max(360, verticalInset * 2 + Math.max(0, rowCount - 1) * 144);
  const nodes = waves.flatMap((wave, column) => wave.map((task, row) => {
    const routingTask = matchingRouting?.tasks.find(candidate => candidate.task_id === task.task_id);
    const selectedAttemptId = routingTask?.current_attempt_id ?? routingTask?.winning_attempt_id;
    const routingAttempt = routingTask?.attempts.find(candidate => candidate.attempt_id === selectedAttemptId) ?? null;
    const matches = routed ? [] : agents.filter(agent => agent.task_id === task.task_id && agent.run_id === run.id && agent.domain_id === run.domain_id);
    const routedAgent = routingAttempt ? exactAttemptAgent(agents, run, task.task_id, routingAttempt.agent_id) : undefined;
    const centeredRow = row + (rowCount - wave.length) / 2;
    return {
      task,
      x: (column + .5) / waves.length * 100,
      y: (row + .5) / wave.length * 100,
      sceneX: orientation === "vertical"
        ? (wave.length === 1 ? width / 2 : horizontalInset + row * 264)
        : (waves.length === 1 ? width / 2 : horizontalInset + column * 320),
      sceneY: orientation === "vertical"
        ? (waves.length === 1 ? height / 2 : verticalInset + column * 154)
        : (rowCount === 1 ? height / 2 : verticalInset + centeredRow * 144),
      agent: routed ? routedAgent : matches.length === 1 ? matches[0] : undefined,
      routingAttempt,
      attemptCount: routingTask?.attempts.length ?? null,
    };
  }));
  const edges = nodes.flatMap(target => target.task.depends_on.flatMap(id => {
    const source = nodes.find(node => node.task.task_id === id);
    return source ? [{ source, target }] : [];
  }));
  return {
    waves, nodes, edges,
    compact: nodes.length > 8 || waves.length > 3 || waves.some(wave => wave.length > 2),
    width,
    height,
    orientation,
  };
}
