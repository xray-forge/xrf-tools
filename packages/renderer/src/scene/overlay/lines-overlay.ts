import { BufferAttribute, BufferGeometry, LineSegments } from "three/webgpu";

import { ERendererOverlay, TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { disposeOverlayObjects, IOverlayDrawing } from "#/scene/overlay/overlay-drawing";
import { createOverlayLines } from "#/scene/overlay/overlay-lines";

/**
 * Line segments coloured per vertex, standing where they were put.
 */
export class LinesOverlay implements IOverlayDrawing {
  public readonly objects: ReadonlyArray<LineSegments>;

  public constructor(overlay: Extract<TRendererOverlay, { kind: ERendererOverlay.LINES }>) {
    const geometry: BufferGeometry = new BufferGeometry();

    geometry.setAttribute("position", new BufferAttribute(overlay.positions, 3));
    geometry.setAttribute("color", new BufferAttribute(overlay.colors, 3));

    this.objects = [createOverlayLines(geometry, overlay.isDepthTested, true)];
  }

  public update(): void {}

  public dispose(): void {
    disposeOverlayObjects(this.objects);
  }
}
