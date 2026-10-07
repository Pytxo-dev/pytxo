export const sampleTasks = [
  { id: 1, title: 'Add keyboard shortcuts', status: 'todo' },
  { id: 2, title: 'Write release notes', status: 'doing' },
  { id: 3, title: 'Test the empty state', status: 'todo' },
  { id: 4, title: 'Review the task layout', status: 'done' },
];

export function addTask(tasks, title) {
  return [...tasks, { id: Math.max(0, ...tasks.map(task => task.id)) + 1, title, status: 'todo' }];
}
