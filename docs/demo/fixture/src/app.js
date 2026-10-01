import { addTask, sampleTasks } from './model.mjs';

let tasks = structuredClone(sampleTasks);
const list = document.querySelector('#tasks');
function render() {
  list.replaceChildren(...tasks.map(task => {
    const row = document.createElement('li');
    const title = document.createElement('span');
    title.textContent = task.title;
    const status = document.createElement('small');
    status.textContent = { todo: 'To do', doing: 'In progress', done: 'Done' }[task.status];
    row.append(title, status);
    return row;
  }));
}
document.querySelector('#new-task').addEventListener('submit', event => {
  event.preventDefault();
  const title = document.querySelector('#title');
  tasks = addTask(tasks, title.value);
  title.value = '';
  render();
});
render();
