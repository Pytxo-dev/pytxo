import test from 'node:test';
import assert from 'node:assert/strict';
import { addTask, sampleTasks } from '../src/model.mjs';

test('sample board has distinct identifiers and useful statuses', () => {
  assert.equal(new Set(sampleTasks.map(task => task.id)).size, 4);
  assert.deepEqual(new Set(sampleTasks.map(task => task.status)), new Set(['todo', 'doing', 'done']));
});
test('adding a task leaves the supplied board unchanged', () => {
  const original = structuredClone(sampleTasks);
  const result = addTask(original, 'Check keyboard focus');
  assert.deepEqual(original, sampleTasks);
  assert.equal(result.length, 5);
  assert.deepEqual(result.at(-1), { id: 5, title: 'Check keyboard focus', status: 'todo' });
});
