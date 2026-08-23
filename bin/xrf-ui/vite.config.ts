import * as path from "path";

import { default as react } from "@vitejs/plugin-react";
import { wirestate } from "@wirestate/dev/vite";
import { default as observerPlugin } from "mobx-react-observer/vite-plugin";
import { defineConfig, Plugin } from "vite";
import { default as inlineSource } from "vite-plugin-inline-source";

import { replaceModuleName } from "./cli/build/module-name";

/**
 * Substitutes `__MODULE_NAME__` with the source file's name, before anything else compiles it.
 *
 * Shares its implementation with `cli/test/transformer.cjs` so a tag reads the same under vite, jest, and a release
 * bundle - which is the whole point, since a bundle keeps neither the module's path nor its class names.
 */
function moduleNamePlugin(): Plugin {
  return {
    name: "xrf-module-name",
    enforce: "pre",
    transform(code, id) {
      const replaced: string = replaceModuleName(code, id);

      return replaced === code ? null : { code: replaced, map: null };
    },
  };
}

function reactObserverPlugin(): Plugin {
  const plugin: Plugin = observerPlugin() as Plugin;
  const transform = plugin.transform;

  return {
    ...plugin,
    name: "mobx-react-observer-tsx",
    transform(code, id) {
      if (id.endsWith(".tsx") && typeof transform === "function") {
        return transform.call(this, code, id);
      }

      return null;
    },
  };
}

function getInitialVendorChunk(id: string): string | null {
  const normalized: string = id.replaceAll("\\", "/");

  if (normalized.includes("/node_modules/")) {
    if (
      normalized.includes("/node_modules/react") ||
      normalized.includes("/node_modules/scheduler") ||
      normalized.includes("/node_modules/@wirestate/")
    ) {
      return "vendor-core";
    }

    if (normalized.includes("/node_modules/@mui/") || normalized.includes("/node_modules/@emotion/")) {
      return "vendor-mui";
    }

    return "vendor";
  }

  return null;
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [moduleNamePlugin(), wirestate(), inlineSource({ optimizeJs: true }), react(), reactObserverPlugin()],
  build: {
    outDir: "target",
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            {
              name: getInitialVendorChunk,
              test: /node_modules[\\/]/,
              tags: ["$initial" as const],
              minSize: 50_000,
            },
          ],
        },
      },
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
});
