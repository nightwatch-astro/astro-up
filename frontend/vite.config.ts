/// <reference types="vitest/config" />
import { readFileSync } from "fs";
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

const host = process.env.TAURI_DEV_HOST;

// Read version from workspace Cargo.toml (source of truth, managed by release-please)
const cargoToml = readFileSync("../Cargo.toml", "utf-8");
const versionMatch = cargoToml.match(/^version\s*=\s*"([^"]+)"/m);
const appVersion = versionMatch ? versionMatch[1] : "0.0.0";

export default defineConfig({
  plugins: [vue()],
  define: {
    __APP_VERSION__: JSON.stringify(appVersion),
  },
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 5174,
        }
      : undefined,
    watch: {
      ignored: ["**/crates/**"],
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
  },
});
