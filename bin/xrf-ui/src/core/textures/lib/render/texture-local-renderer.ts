import { TFrameRateLimit } from "@xrf/renderer";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import {
  ITextureSurfaceRenderer,
  ITextureSurfaceRendererEvents,
} from "@/core/textures/lib/render/texture-surface-renderer";
import { TextureSurfaceScene } from "@/core/textures/lib/scene/TextureSurfaceScene";
import { ITextureSurfaceFiles, ITextureSurfaceOptions } from "@/core/textures/lib/texture-surface";

/** What a renderer needs to exist at all: somewhere to draw, and somewhere to report to. */
export interface ITextureLocalRendererOptions {
  target: DomRenderTarget;
  events: ITextureSurfaceRendererEvents;
}

/**
 * Draws the lit surface on the thread that asked.
 */
export class TextureLocalRenderer implements ITextureSurfaceRenderer {
  private readonly scene: TextureSurfaceScene;
  private readonly events: ITextureSurfaceRendererEvents;

  public constructor({ target, events }: ITextureLocalRendererOptions) {
    this.events = events;
    // On the canvas rather than the element around it: it is what the controls listen to and capture on.
    this.scene = new TextureSurfaceScene(target, target.canvas);

    this.scene.setReporter((cost) => events.onReport(cost));
  }

  public setTextures(files: ITextureSurfaceFiles): void {
    this.scene.setTextures(files);
  }

  public setOptions(options: ITextureSurfaceOptions): void {
    this.scene.setOptions(options);
  }

  public setLighting(lighting: IRenderLighting): void {
    this.scene.setLighting(lighting);
  }

  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.scene.setFrameRateLimit(limit);
  }

  public dragLight(deltaX: number, deltaY: number): void {
    this.events.onLighting(this.scene.dragLight(deltaX, deltaY));
  }

  public dolly(step: number): void {
    this.scene.dolly(step);
  }

  public reset(): void {
    this.scene.reset();
  }

  public dispose(): void {
    this.scene.setReporter(null);
    this.scene.dispose();
  }
}
