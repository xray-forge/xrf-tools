import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderWorkerClient } from "@/core/render/lib/worker/render-worker-client";
import {
  ETextureSurfaceRequest,
  ETextureSurfaceResponse,
  TTextureSurfaceRequest,
  TTextureSurfaceResponse,
} from "@/core/textures/lib/render/texture-surface-messages";
import {
  ITextureSurfaceRenderer,
  ITextureSurfaceRendererEvents,
} from "@/core/textures/lib/render/texture-surface-renderer";
import { ITextureSurfaceFiles, ITextureSurfaceOptions } from "@/core/textures/lib/texture-surface";
import { Logger } from "@/lib/logging";

/** What a renderer on another thread needs: somewhere to draw, and somewhere to report to. */
export interface ITextureWorkerRendererOptions {
  target: DomRenderTarget;
  events: ITextureSurfaceRendererEvents;
}

/**
 * Draws the lit surface on a thread of its own.
 */
export class TextureWorkerRenderer implements ITextureSurfaceRenderer {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly client: RenderWorkerClient<TTextureSurfaceRequest, TTextureSurfaceResponse>;
  private readonly events: ITextureSurfaceRendererEvents;

  public constructor({ target, events }: ITextureWorkerRendererOptions) {
    this.events = events;
    this.client = new RenderWorkerClient({
      log: this.log,
      onResponse: (response: TTextureSurfaceResponse): void => this.receive(response),
      target,
      worker: new Worker(new URL("./texture-surface.worker.ts", import.meta.url), { type: "module" }),
    });
  }

  public setTextures(files: ITextureSurfaceFiles): void {
    this.client.post({ files, kind: ETextureSurfaceRequest.TEXTURES });
  }

  public setOptions(options: ITextureSurfaceOptions): void {
    this.client.post({ kind: ETextureSurfaceRequest.OPTIONS, options });
  }

  public setLighting(lighting: IRenderLighting): void {
    this.client.post({ kind: ETextureSurfaceRequest.LIGHTING, lighting });
  }

  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.client.post({ kind: ETextureSurfaceRequest.FRAME_RATE, limit });
  }

  public dragLight(deltaX: number, deltaY: number): void {
    this.client.post({ deltaX, deltaY, kind: ETextureSurfaceRequest.DRAG_LIGHT });
  }

  public dolly(step: number): void {
    this.client.post({ kind: ETextureSurfaceRequest.DOLLY, step });
  }

  public reset(): void {
    this.client.post({ kind: ETextureSurfaceRequest.RESET });
  }

  public dispose(): void {
    this.client.dispose();
  }

  private receive(response: TTextureSurfaceResponse): void {
    switch (response.kind) {
      case ETextureSurfaceResponse.LIGHTING:
        return this.events.onLighting(response.lighting);

      case ETextureSurfaceResponse.REPORT:
        return this.events.onReport(response.cost);
    }
  }
}
