import path from "node:path";
import { fileURLToPath } from "node:url";

import autoprefixer from "autoprefixer";
import tailwindcss from "tailwindcss";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  envDir: path.resolve(__dirname, "../.."),
  plugins: [react()],
  css: {
    postcss: {
      plugins: [tailwindcss(), autoprefixer()],
    },
  },
  server: {
    port: 5174,
    strictPort: true,
    fs: {
      // Разрешаем импортировать код из соседних app'ов монорепо (например, obs-overlay компоненты).
      allow: [".."],
    },
    proxy: {
      // Опционально: dev-бэкенд на :8000
      "/api": "http://localhost:8000",
      "/ws": {
        target: "ws://localhost:8000",
        ws: true,
      },
    },
  },
});

