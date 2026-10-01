import { Camera, Object3D, Scene, WebGPURenderer } from "three/webgpu";

import { drawUnsorted } from "#/pass/unsorted-draw";

/**
 * Draws several roots in one render call, one pass, each put under a holder of the caller's own for the call and taken
 * out after: every render call is a command encoder, a command buffer and a pass, which the GPU process frees only
 * once the worker's next major collection lets their wrappers go, all at once. Drawn unsorted, in the order given, into
 * the target current, with three's own clears as the renderer has them.
 *
 * @param renderer - The renderer drawing.
 * @param holder - A scene of the caller's own, holding nothing else, with no background.
 * @param parts - What is drawn, in order: scenes, or objects drawn on their own.
 * @param camera - What they are drawn with.
 */
export function drawTogether(
  renderer: WebGPURenderer,
  holder: Scene,
  parts: ReadonlyArray<Object3D>,
  camera: Camera
): void {
  for (const part of parts) {
    holder.add(part);
  }

  try {
    drawUnsorted(renderer, () => renderer.render(holder, camera));
  } finally {
    for (const part of parts) {
      holder.remove(part);
    }
  }
}
