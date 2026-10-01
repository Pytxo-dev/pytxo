import { expect, test } from "@playwright/test";
import { recordedCliFor } from "../src/lib/review-state";
import type { AgentDto } from "../src/lib/types";

const run = { id: "run-1", domain_id: "d" };
const agent = (task: string, cli: string | null, name = cli ?? ""): AgentDto => ({
  id: `run-1:${task}`, domain_id: "d", run_id: "run-1", task_id: task, wave: 0, status: "completed", exit_code: 0,
  root_id: null, launcher: cli ? { id: cli, display_name: name } : null,
} as AgentDto);

test("Review names the CLI that prepared a file only in mixed-CLI runs", () => {
  const mixed = [agent("style", "claude", "Claude Code"), agent("model", "codex", "OpenAI Codex")];
  expect(recordedCliFor(mixed, run, "style")).toBe("Claude Code");
  expect(recordedCliFor(mixed, run, "model")).toBe("OpenAI Codex");
  expect(recordedCliFor(mixed, run, "missing")).toBeNull();
  // One CLI for the whole run: naming it on every file adds nothing.
  expect(recordedCliFor([agent("a", "codex"), agent("b", "codex")], run, "a")).toBeNull();
  // A wrapped or modified command stays unknown rather than guessed.
  expect(recordedCliFor([...mixed, agent("custom", null)], run, "custom")).toBeNull();
});
