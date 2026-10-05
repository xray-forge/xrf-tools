import * as path from "path";

import { default as tailwindcss } from "@tailwindcss/vite";
import { default as react } from "@vitejs/plugin-react";
import { wirestate } from "@wirestate/dev/vite";
import { defineConfig, normalizePath, type Plugin, runnerImport } from "vite";
import { default as inlineSource } from "vite-plugin-inline-source";

import { replaceModuleName } from "./cli/build/module-name.ts";
import { applyObserver } from "./cli/build/observer.ts";
import { default as manifest } from "./package.json" with { type: "json" };

/**
 * Inlines the first-paint theme stylesheet into `index.html`.
 */
function preloadThemePlugin(): Plugin {
  const entry: string = normalizePath(path.resolve(import.meta.dirname, "./src/core/theme/preload.ts"));
  let files: ReadonlySet<string> = new Set([entry]);
  let css = null;

  async function load(): Promise<string> {
    const { module, dependencies } = await runnerImport<typeof import("./src/core/theme/preload.ts")>(entry);

    files = new Set([entry, ...dependencies.map(normalizePath)]);

    return module.getPreloadThemeCss();
  }

  return {
    name: "xrf-preload-theme",
    configureServer(server) {
      // The stylesheet's variables outlive the first paint, so a changed theme needs a fresh page, not just HMR.
      server.watcher.on("change", (file: string) => {
        if (files.has(normalizePath(file))) {
          css = null;
          server.ws.send({ type: "full-reload" });
        }
      });
    },
    transformIndexHtml: {
      order: "pre",
      handler: async () => {
        css ??= load().catch((error: unknown) => {
          css = null;
          throw error;
        });

        return [{ tag: "style", children: await css, injectTo: "head" }];
      },
    },
  };
}

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
  return {
    name: "mobx-react-observer-tsx",
    enforce: "pre",
    transform(code, id) {
      const transformed: string = applyObserver(code, id);

      return transformed === code ? null : { code: transformed, map: null };
    },
  };
}

function getInitialVendorChunk(id: string) {
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
  root: path.resolve(import.meta.dirname, "./src"),
  publicDir: path.resolve(import.meta.dirname, "./public"),
  cacheDir: path.resolve(import.meta.dirname, "./node_modules/.vite"),
  define: {
    __REPOSITORY_URL__: JSON.stringify(manifest.repository.url),
  },
  plugins: [
    moduleNamePlugin(),
    preloadThemePlugin(),
    wirestate(),
    tailwindcss(),
    inlineSource({ optimizeJs: true }),
    react(),
    reactObserverPlugin(),
  ],
  build: {
    // Beside, not over, the tool caches in `target/`: Tauri embeds this whole directory and this build empties it.
    outDir: path.resolve(import.meta.dirname, "./target/dist"),
    emptyOutDir: true,
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
      "@": path.resolve(import.meta.dirname, "./src"),
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
