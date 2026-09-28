import { Sprite, SpriteNodeMaterial, Vector3 } from "three/webgpu";

import { ERendererOverlay, TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { disposeOverlayObjects, IOverlayDrawing, OVERLAY_RENDER_ORDER } from "#/scene/overlay/overlay-drawing";
import { IOverlayFrame } from "#/scene/overlay/overlay-frame";
import { toSunDiscOpacity } from "#/scene/overlay/sun-overlay.tsl";

/** How far out the marker stands, as a share of the far plane: well inside it, and past whatever is near. */
const SUN_REACH: number = 0.5;

/**
 * A disc in the sky where the light comes from, following the camera and the lighting.
 */
export class SunOverlay implements IOverlayDrawing {
  public readonly objects: ReadonlyArray<Sprite>;

  private readonly sun: Sprite;
  private readonly size: number;
  /** The direction sunlight travels, in world space, read each frame. */
  private readonly direction: Vector3;

  public constructor(overlay: Extract<TRendererOverlay, { kind: ERendererOverlay.SUN }>, direction: Vector3) {
    const material: SpriteNodeMaterial = new SpriteNodeMaterial({ sizeAttenuation: false });

    material.color.setRGB(...overlay.color);
    material.opacityNode = toSunDiscOpacity();
    material.alphaTest = 0.5;
    material.depthTest = false;
    material.depthWrite = false;

    this.size = overlay.size;
    this.direction = direction;
    this.sun = new Sprite(material);
    this.sun.renderOrder = OVERLAY_RENDER_ORDER;
    // Placed every frame, so never measured against where it last stood.
    this.sun.frustumCulled = false;
    this.objects = [this.sun];
  }

  public update({ camera, pixel }: IOverlayFrame): void {
    // Against the light's travel from wherever the camera stands, as a sun far enough away does.
    this.sun.position.copy(camera.position).addScaledVector(this.direction, -camera.far * SUN_REACH);
    this.sun.scale.setScalar(this.size * pixel);
  }

  public dispose(): void {
    disposeOverlayObjects(this.objects);
  }
}
