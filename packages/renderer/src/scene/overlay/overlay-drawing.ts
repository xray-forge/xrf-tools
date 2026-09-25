import { LineSegments, Object3D, Sprite } from "three/webgpu";

import { IOverlayFrame } from "#/scene/overlay/overlay-frame";

/** Drawn after every surface, so a helper that ignores depth is never covered by one drawn later. */
export const OVERLAY_RENDER_ORDER: number = 1;

/**
 * One overlay as three draws it.
 */
export interface IOverlayDrawing {
  /** What stands in the overlay pass's scene for it. */
  readonly objects: ReadonlyArray<Object3D>;
  /**
   * Brings it up to date for the frame about to be drawn.
   *
   * @param frame - What it follows.
   */
  update(frame: IOverlayFrame): void;
  dispose(): void;
}

/**
 * @param objects - An overlay's objects, taken out of their scene and let go of on the device. A sprite's geometry is
 *   the one quad three shares between every sprite, so only its material goes: disposing the quad destroys its buffers
 *   under every sprite put after, and the overlay pass that draws one is dropped whole, grid and all.
 */
export function disposeOverlayObjects(objects: ReadonlyArray<Object3D>): void {
  for (const object of objects) {
    object.removeFromParent();

    if (object instanceof LineSegments) {
      object.geometry.dispose();
      (object.material as { dispose(): void }).dispose();
    } else if (object instanceof Sprite) {
      (object.material as { dispose(): void }).dispose();
    }
  }
}
