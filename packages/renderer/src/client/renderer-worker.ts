/**
 * Starts the renderer's worker.
 *
 * Exported as `xrf-renderer/worker` rather than from the package root: `import.meta.url` does not survive a CommonJS
 * test transform, so a consumer imports this from a module its tests replace.
 *
 * @returns The worker, ready for a `RendererClient`.
 */
export function createRendererWorker(): Worker {
  return new Worker(new URL("../host/renderer.worker.ts", import.meta.url), { type: "module" });
}
