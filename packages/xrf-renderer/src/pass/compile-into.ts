import { Nullable } from "@xrf/types";
import { Camera, Object3D, RenderTarget, Scene, WebGPURenderer } from "three/webgpu";

/**
 * Compiles what an object draws with for a target. Three takes the target as the compile starts and builds each shader
 * later for the render context it took then, so the target current before is restored at once. It gives that context
 * the renderer's own depth and stencil where a draw gives it the target's, so the compile is handed the target's: a
 * pipeline built with a depth its target lacks is one no draw uses.
 *
 * @param renderer - The renderer drawing.
 * @param target - Where the object draws: the canvas when null.
 * @param object - What compiles: a scene, or one object drawn on its own.
 * @param camera - What it draws with.
 * @param scene - The scene an object is drawn in, whose lighting and environment its materials build with; none for
 *   a scene itself, or an object drawn on its own.
 * @returns Settles once every pipeline is built.
 */
export function compileInto(
  renderer: WebGPURenderer,
  target: Nullable<RenderTarget>,
  object: Object3D,
  camera: Camera,
  scene: Nullable<Scene> = null
): Promise<void> {
  const previous: Nullable<RenderTarget> = renderer.getRenderTarget();
  const { depth, stencil } = renderer;

  renderer.setRenderTarget(target);

  if (target) {
    renderer.depth = target.depthBuffer;
    renderer.stencil = target.stencilBuffer;
  }

  const compiled: Promise<void> = renderer.compileAsync(object, camera, scene);

  renderer.depth = depth;
  renderer.stencil = stencil;
  renderer.setRenderTarget(previous);

  return compiled;
}
