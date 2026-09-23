import { Sprite, SpriteNodeMaterial } from "three/webgpu";

import { ERendererOverlay, TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { disposeOverlayObjects, IOverlayDrawing, OVERLAY_RENDER_ORDER } from "#/scene/overlay/overlay-drawing";
import { IOverlayFrame } from "#/scene/overlay/overlay-frame";

/**
 * Points of one colour, a fixed number of pixels across wherever they stand.
 */
export class PointsOverlay implements IOverlayDrawing {
  public readonly objects: ReadonlyArray<Sprite>;

  private readonly size: number;

  public constructor(overlay: Extract<TRendererOverlay, { kind: ERendererOverlay.POINTS }>) {
    this.size = overlay.size;
    this.objects = Array.from({ length: overlay.positions.length / 3 }, (_, at: number) => {
      const material: SpriteNodeMaterial = new SpriteNodeMaterial({ sizeAttenuation: false });
      const sprite: Sprite = new Sprite(material);

      material.color.setRGB(...overlay.color);
      material.depthTest = overlay.isDepthTested;
      material.depthWrite = false;
      sprite.position.fromArray(overlay.positions, at * 3);
      sprite.renderOrder = OVERLAY_RENDER_ORDER;

      return sprite;
    });
  }

  public update({ pixel }: IOverlayFrame): void {
    this.objects.forEach((sprite: Sprite) => sprite.scale.setScalar(this.size * pixel));
  }

  public dispose(): void {
    disposeOverlayObjects(this.objects);
  }
}
