import { WebGPURenderer } from "three/webgpu";

/**
 * Draws with what is asked cleared as the pass begins, rather than by a clear of its own first: a clear is a pass, and
 * every pass is objects the GPU process frees only once the worker's next major collection lets their wrappers go, all
 * at once. The same as a clear and then the draw, so long as the draw is one render call.
 *
 * @param renderer - The renderer drawing, its target and clear values current.
 * @param isColor - Whether the colour is cleared.
 * @param isDepth - Whether the depth is.
 * @param draw - Draws the pass's one render call.
 */
export function drawCleared(renderer: WebGPURenderer, isColor: boolean, isDepth: boolean, draw: () => void): void {
  const { autoClear, autoClearColor, autoClearDepth, autoClearStencil } = renderer;

  renderer.autoClear = true;
  renderer.autoClearColor = isColor;
  renderer.autoClearDepth = isDepth;
  renderer.autoClearStencil = false;

  try {
    draw();
  } finally {
    renderer.autoClear = autoClear;
    renderer.autoClearColor = autoClearColor;
    renderer.autoClearDepth = autoClearDepth;
    renderer.autoClearStencil = autoClearStencil;
  }
}
