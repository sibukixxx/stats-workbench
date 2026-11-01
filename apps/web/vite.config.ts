import { defineConfig } from "vitest/config";
import preact from "@preact/preset-vite";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [preact(), tailwindcss()],
  server: { allowedHosts: ["terminal.local"] },
  test: {
    environment: "jsdom",
    setupFiles: "./src/test/setup.ts",
    css: true,
  },
});
