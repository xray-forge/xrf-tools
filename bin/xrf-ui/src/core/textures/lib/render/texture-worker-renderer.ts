import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IOffscreenRenderSize } from "@/core/render/lib/frame/offscreen-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { IRenderInputEvent } from "@/core/render/lib/worker/render-input";
import { RenderInputForwarder } from "@/core/render/lib/worker/render-input-forwarder";
import {
  ETextureSurfaceRequest,
  ETextureSurfaceResponse,
  listTextureSurfaceTransfers,
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

  private readonly worker: Worker;
  private readonly target: DomRenderTarget;
  private readonly events: ITextureSurfaceRendererEvents;
  private readonly input: RenderInputForwarder;
  private readonly unobserve: () => void;

  public constructor({ target, events }: ITextureWorkerRendererOptions) {
    this.target = target;
    this.events = events;

    this.worker = new Worker(new URL("./texture-surface.worker.ts", import.meta.url), { type: "module" });
    this.worker.onmessage = (event: MessageEvent<TTextureSurfaceResponse>): void => this.receive(event.data);
    this.worker.onerror = (event: ErrorEvent): void =>
      this.log.error("The texture surface worker failed:", event.message);

    this.log.info("Started a texture surface worker");

    this.post({
      canvas: target.canvas.transferControlToOffscreen(),
      kind: ETextureSurfaceRequest.START,
      ...this.getSize(),
    });

    this.input = new RenderInputForwarder(target.canvas, (event: IRenderInputEvent) =>
      this.post({ event, kind: ETextureSurfaceRequest.INPUT })
    );
    this.unobserve = target.observe(() => this.post({ kind: ETextureSurfaceRequest.RESIZE, ...this.getSize() }));
  }

  public setTextures(files: ITextureSurfaceFiles): void {
    this.post({ files, kind: ETextureSurfaceRequest.TEXTURES });
  }

  public setOptions(options: ITextureSurfaceOptions): void {
    this.post({ kind: ETextureSurfaceRequest.OPTIONS, options });
  }

  public setLighting(lighting: IRenderLighting): void {
    this.post({ kind: ETextureSurfaceRequest.LIGHTING, lighting });
  }

  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.post({ kind: ETextureSurfaceRequest.FRAME_RATE, limit });
  }

  public dragLight(deltaX: number, deltaY: number): void {
    this.post({ deltaX, deltaY, kind: ETextureSurfaceRequest.DRAG_LIGHT });
  }

  public dolly(step: number): void {
    this.post({ kind: ETextureSurfaceRequest.DOLLY, step });
  }

  public reset(): void {
    this.post({ kind: ETextureSurfaceRequest.RESET });
  }

  public dispose(): void {
    this.unobserve();
    this.input.dispose();

    // Terminating reclaims the thread whole, context and all; the message is what releases a server that is
    // not on a worker, and is said first so the two are released the same way.
    this.post({ kind: ETextureSurfaceRequest.DISPOSE });
    this.worker.terminate();

    this.log.info("Terminated the texture surface worker");
  }

  private post(request: TTextureSurfaceRequest): void {
    this.worker.postMessage(request, listTextureSurfaceTransfers(request));
  }

  private receive(response: TTextureSurfaceResponse): void {
    switch (response.kind) {
      case ETextureSurfaceResponse.LIGHTING:
        return this.events.onLighting(response.lighting);

      case ETextureSurfaceResponse.REPORT:
        return this.events.onReport(response.cost);

      case ETextureSurfaceResponse.CURSOR:
        return this.input.setCursor(response.cursor);
    }
  }

  private getSize(): IOffscreenRenderSize {
    return { height: this.target.height, pixelRatio: this.target.pixelRatio, width: this.target.width };
  }
}
