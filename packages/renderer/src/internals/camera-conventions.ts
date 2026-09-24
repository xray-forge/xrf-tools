import { PerspectiveCamera, WebGPUCoordinateSystem } from "three/webgpu";

/**
 * Brings a camera to the renderer's conventions, WebGPU clip space and reversed depth, and rebuilds its projection.
 * Three does the same inside `render`, after the cull has taken the camera's frustum: a camera made without this is
 * culled against a conventional projection for its first frame.
 *
 * @param camera - A camera the renderer draws with, before anything reads it.
 */
export function adoptRendererConventions(camera: PerspectiveCamera): void {
  camera.coordinateSystem = WebGPUCoordinateSystem;
  // Private in three, behind a getter: `reversedDepth` has no setter.
  (camera as unknown as { _reversedDepth: boolean })._reversedDepth = true;
  camera.updateProjectionMatrix();
}
