import { PerspectiveCamera } from "three/webgpu";

import { IRendererCameraLens } from "#/contract/renderer-camera-lens";

/**
 * @param camera - The camera a controller drives.
 * @param lens - What the consumer described it seeing through.
 */
export function setCameraLens(camera: PerspectiveCamera, lens: IRendererCameraLens): void {
  camera.fov = lens.fieldOfView;
  camera.near = lens.near;
  camera.far = lens.far;
  camera.updateProjectionMatrix();
}

/**
 * @param camera - The camera a controller drives.
 * @param width - Drawing width, in any unit the height shares.
 * @param height - Drawing height.
 */
export function setCameraAspect(camera: PerspectiveCamera, width: number, height: number): void {
  const aspect: number = width / Math.max(height, 1);

  if (aspect !== camera.aspect) {
    camera.aspect = aspect;
    camera.updateProjectionMatrix();
  }
}
