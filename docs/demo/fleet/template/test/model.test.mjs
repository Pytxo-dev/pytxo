import assert from "node:assert/strict";
import test from "node:test";
import { addTask, sampleTasks, toggleTask } from "../src/model.mjs";

test("adding a task gives it the next id and an open status", () => {
  const tasks = addTask(sampleTasks, "Plan the offsite");
  assert.deepEqual(tasks.at(-1), { id: 5, title: "Plan the offsite", status: "open" });
});

test("toggling flips one task and leaves the input list alone", () => {
  const tasks = toggleTask(sampleTasks, 1);
  assert.equal(tasks[0].status, "done");
  assert.equal(sampleTasks[0].status, "open");
});
