import { defineConfig } from "vite";
import { resolve } from "node:path";

export default defineConfig({
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
  },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "esnext",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    outDir: "dist",
    // Two entry points: main control window + stage display companion.
    // Each renders independently in its own Tauri webview.
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, "index.html"),
        stage: resolve(import.meta.dirname, "stage.html"),
      },
    },
  },
});
