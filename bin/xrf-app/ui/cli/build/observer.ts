import { type PluginObject, transformSync } from "@babel/core";
import createObserverPlugin from "babel-plugin-observing-components";

/**
 * Applies the same component-observer transform in Vite and Jest.
 * Babel parses decorators here; the subsequent compiler owns their runtime semantics.
 *
 * @param source - Module source before compilation.
 * @param filename - Module path used to select component files.
 * @returns Source with component observers, or the unchanged non-component source.
 */
export function applyObserver(source: string, filename: string): string {
  if (!filename.endsWith(".tsx") || filename.includes("node_modules")) {
    return source;
  }

  const [observer, options] = createObserverPlugin({ importPath: "mobx-react-observer" });
  // The plugin still declares Babel 7 AST types; both build paths exercise it with Babel 8.
  const observerPlugin = observer as unknown as PluginObject;
  const result = transformSync(source, {
    babelrc: false,
    configFile: false,
    filename,
    parserOpts: { plugins: ["jsx", "typescript", "decorators-legacy"] },
    plugins: [[() => observerPlugin, options]],
  });

  return result?.code ?? source;
}
