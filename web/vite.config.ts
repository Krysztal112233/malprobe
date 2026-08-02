import { fileURLToPath, URL } from "node:url";

import vue from "@vitejs/plugin-vue";
import { defineConfig } from "vite";

// The backend listens on 127.0.0.1:8000 locally. Proxying API routes through
// the dev server keeps the frontend same-origin, so no CORS config is needed
// during development.
const backend = process.env.MALPROBE_BACKEND ?? "http://127.0.0.1:8000";

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
  server: {
    port: 5173,
    proxy: {
      "/files": backend,
      "/docs": backend,
      "/openapi.json": backend,
    },
  },
});
