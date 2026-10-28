import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// `just studio-dev` sets CK_PORT to the running `ck studio`. The server checks Host and Origin,
// so the dev proxy presents itself as the server's own origin.
const port = process.env.CK_PORT;
const target = port ? `http://127.0.0.1:${port}` : undefined;

export default defineConfig({
  plugins: [react()],
  base: "/",
  build: {
    outDir: "dist",
    emptyOutDir: true,
    // Served from the local binary, never over a network: one bundle is fine.
    chunkSizeWarningLimit: 2048,
    sourcemap: false,
  },
  server: target
    ? {
        proxy: {
          "/api": { target, changeOrigin: true, headers: { origin: target } },
          "/ws": {
            target,
            changeOrigin: true,
            ws: true,
            headers: { origin: target },
          },
        },
      }
    : undefined,
});
