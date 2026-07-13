import "../src/app.css";
import type { Preview } from "@storybook/svelte-vite";

const preview: Preview = {
  parameters: {
    layout: "fullscreen",
    a11y: { test: "error" },
    viewport: {
      options: {
        desktop1600: { name: "Desktop 1600", styles: { width: "1600px", height: "1000px" } },
        desktop1280: { name: "Desktop 1280", styles: { width: "1280px", height: "800px" } },
        compact960: { name: "Compact 960", styles: { width: "960px", height: "640px" } },
      },
    },
  },
};

export default preview;
