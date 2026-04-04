import App from "./App.svelte";
import "@fontsource-variable/plus-jakarta-sans";
import "@fontsource/instrument-serif/latin-400.css";
import "@fontsource/instrument-serif/latin-400-italic.css";
import "./app.css";
import { mount } from "svelte";

const app = mount(App, {
  target: document.getElementById("app")!,
});

export default app;
