// Task board model: plain data in, plain data out.
export const STATUSES = ["open", "done"];

export const sampleTasks = [
  { id: 1, title: "Write the release notes", status: "open" },
  { id: 2, title: "Fix the login redirect", status: "done" },
  { id: 3, title: "Review the dashboard copy", status: "open" },
  { id: 4, title: "Archive old invoices", status: "done" },
];

export function addTask(tasks, title) {
  const id = Math.max(0, ...tasks.map((task) => task.id)) + 1;
  return [...tasks, { id, title, status: "open" }];
}

export function toggleTask(tasks, id) {
  return tasks.map((task) =>
    task.id === id ? { ...task, status: task.status === "open" ? "done" : "open" } : task,
  );
}
