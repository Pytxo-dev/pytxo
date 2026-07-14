import "./app.css";
import App from "./App.svelte";
import { mount } from "svelte";
import { initUiPrefs } from "./lib/ui-prefs.svelte";

initUiPrefs();

const app = mount(App, { target: document.getElementById("app")! });
export default app;
