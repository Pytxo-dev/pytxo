import { addTask, sampleTasks, toggleTask } from "./model.mjs";

let tasks = sampleTasks;
const list = document.querySelector("#tasks");
const form = document.querySelector("#new-task");

function render() {
  list.replaceChildren(
    ...tasks.map((task) => {
      const item = document.createElement("li");
      item.className = task.status;
      const box = document.createElement("input");
      box.type = "checkbox";
      box.checked = task.status === "done";
      box.addEventListener("change", () => {
        tasks = toggleTask(tasks, task.id);
        render();
      });
      const label = document.createElement("span");
      label.textContent = task.title;
      item.append(box, label);
      return item;
    }),
  );
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  const input = form.elements.namedItem("title");
  tasks = addTask(tasks, input.value);
  input.value = "";
  render();
});

render();
