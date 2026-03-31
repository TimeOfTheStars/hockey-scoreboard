import path from "node:path";
import { fileURLToPath } from "node:url";

import autoprefixer from "autoprefixer";
import tailwindcss from "tailwindcss";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
  // Только env из каталога оверлея — корневой `.env` монорепо не должен вшивать внешний API в gateway.
  envDir: path.resolve(__dirname, "."),
  plugins: [react()],
  css: {
    postcss: {
      plugins: [tailwindcss(), autoprefixer()],
    },
  },
  server: {
    proxy: {
      // На этапе разработки может быть полезно направить запросы на текущий backend (:8000)
      "/api": "http://localhost:8000",
      "/logos": "http://localhost:8000",
      "/ws": {
        target: "ws://localhost:8000",
        ws: true,
      },
    },
  },
});

