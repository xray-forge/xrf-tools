import { RenderTarget, Vector2 } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";

/**
 * Draws the helpers over the finished frame: unlit, as the colours they name, from the view unjittered, tested against
 * the frame's depth where they ask to be.
 */
export class OverlayPass implements IRendererPass {
  public readonly name: string = "overlay";

  private readonly overlays: RendererOverlays;
  private readonly size: Vector2 = new Vector2();
  /** The frame the helpers draw over, with a depth of its size. */
  private target: RenderTarget;

  /**
   * @param overlays - The helpers.
   * @param target - The frame they draw over until another is set.
   */
  public constructor(overlays: RendererOverlays, target: RenderTarget) {
    this.overlays = overlays;
    this.target = target;
  }

  /**
   * @param target - The frame the helpers draw over from now on, with a depth of its size.
   */
  public setTarget(target: RenderTarget): void {
    this.target = target;
  }

  /** The frame the helpers draw over now. */
  public get frame(): RenderTarget {
    return this.target;
  }

  public render({ renderer, viewCamera }: IRendererFrame): void {
    renderer.getDrawingBufferSize(this.size);
    this.overlays.update(viewCamera, this.size.y);
    renderer.setRenderTarget(this.target);
    renderer.render(this.overlays.scene, viewCamera);
  }

  /** The overlays are the host's, which lets them go with the rest of what the consumer put. */
  public dispose(): void {}
}
