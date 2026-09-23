import { Maybe } from "@xrf/types";
import { Object3D, PerspectiveCamera, Scene, Vector3 } from "three/webgpu";

import { ERendererOverlay, TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { LinesOverlay } from "#/scene/overlay/lines-overlay";
import { IOverlayDrawing } from "#/scene/overlay/overlay-drawing";
import { IOverlayFrame } from "#/scene/overlay/overlay-frame";
import { PointsOverlay } from "#/scene/overlay/points-overlay";
import { SkeletonOverlay } from "#/scene/overlay/skeleton-overlay";
import { SunOverlay } from "#/scene/overlay/sun-overlay";
import { RendererSkeletons } from "#/scene/skeleton/renderer-skeletons";

/**
 * The helpers a consumer put, by key, as the scene the overlay pass draws.
 */
export class RendererOverlays {
  public readonly scene: Scene = new Scene();

  private readonly drawings: Map<string, IOverlayDrawing> = new Map();
  private readonly skeletons: RendererSkeletons;
  /** The direction sunlight travels, in world space. */
  private readonly sunDirection: Vector3;

  public constructor(skeletons: RendererSkeletons, sunDirection: Vector3) {
    this.skeletons = skeletons;
    this.sunDirection = sunDirection;
  }

  public put(key: string, overlay: TRendererOverlay): void {
    this.release(key);

    const drawing: IOverlayDrawing = this.create(overlay);

    drawing.objects.forEach((object: Object3D) => this.scene.add(object));
    this.drawings.set(key, drawing);
  }

  public release(key: string): void {
    const drawing: Maybe<IOverlayDrawing> = this.drawings.get(key);

    if (drawing) {
      drawing.dispose();
      this.drawings.delete(key);
    }
  }

  /**
   * Brings every overlay up to date for the frame about to be drawn.
   *
   * @param camera - The drawing camera, which a point's pixel size is measured against.
   * @param height - The drawing buffer's height, in device pixels.
   */
  public update(camera: PerspectiveCamera, height: number): void {
    // A sprite without attenuation is scaled by its depth, so this is pixels over the view's height at depth one.
    const frame: IOverlayFrame = { camera, pixel: (2 * Math.tan((camera.fov * Math.PI) / 360)) / Math.max(height, 1) };

    this.drawings.forEach((drawing: IOverlayDrawing) => drawing.update(frame));
  }

  public dispose(): void {
    this.drawings.forEach((drawing: IOverlayDrawing) => drawing.dispose());
    this.drawings.clear();
  }

  private create(overlay: TRendererOverlay): IOverlayDrawing {
    switch (overlay.kind) {
      case ERendererOverlay.LINES:
        return new LinesOverlay(overlay);

      case ERendererOverlay.POINTS:
        return new PointsOverlay(overlay);

      case ERendererOverlay.SKELETON:
        return new SkeletonOverlay(overlay, this.skeletons);

      case ERendererOverlay.SUN:
        return new SunOverlay(overlay, this.sunDirection);
    }
  }
}
