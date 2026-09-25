import { Nullable } from "@xrf/types";
import { RenderTarget, Vector2 } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";

/**
 * Draws the helpers over the composited frame: unlit, as the colours they name, tested against the G-buffer's depth
 * where they ask to be.
 */
export class OverlayPass implements IRendererPass {
  public readonly name: string = "overlay";

  private readonly overlays: RendererOverlays;
  private readonly size: Vector2 = new Vector2();
  /** Where the helpers draw instead of the composited frame, once a pass resolved it into a target of its own. */
  private target: Nullable<RenderTarget> = null;

  public constructor(overlays: RendererOverlays) {
    this.overlays = overlays;
  }

  /**
   * @param target - The frame the helpers draw over from now on, with the G-buffer's depth; null for the composited one.
   */
  public setTarget(target: Nullable<RenderTarget>): void {
    this.target = target;
  }

  public render({ renderer, camera, targets }: IRendererFrame): void {
    renderer.getDrawingBufferSize(this.size);
    this.overlays.update(camera, this.size.y);
    renderer.setRenderTarget(this.target ?? targets.composite);
    renderer.render(this.overlays.scene, camera);
  }

  /** The overlays are the host's, which lets them go with the rest of what the consumer put. */
  public dispose(): void {}
}
