import { OffscreenRenderTarget } from "@/core/render/lib/frame/offscreen-render-target";
import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import {
  ETextureSurfaceRequest,
  ETextureSurfaceResponse,
  TTextureSurfaceRequest,
  TTextureSurfaceResponse,
} from "@/core/textures/lib/render/texture-surface-messages";
import { TextureSurfaceScene } from "@/core/textures/lib/scene/TextureSurfaceScene";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Where a served surface sends what it has to say. */
export type TTextureSurfaceReply = (response: TTextureSurfaceResponse) => void;

/**
 * Draws a lit surface for somebody else's thread.
 */
export class TextureSurfaceServer {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly reply: TTextureSurfaceReply;

  private scene: Nullable<TextureSurfaceScene> = null;
  private target: Nullable<OffscreenRenderTarget> = null;
  private element: Nullable<RenderProxyElement> = null;

  public constructor(reply: TTextureSurfaceReply) {
    this.reply = reply;
  }

  /**
   * Takes one message.
   *
   * @param request - What the page said.
   */
  public take(request: TTextureSurfaceRequest): void {
    switch (request.kind) {
      case ETextureSurfaceRequest.START:
        return this.start(request);

      case ETextureSurfaceRequest.RESIZE:
        this.target?.resize(request);
        this.element?.resize(request);

        return;

      case ETextureSurfaceRequest.TEXTURES:
        return this.scene?.setTextures(request.files);

      case ETextureSurfaceRequest.OPTIONS:
        return this.scene?.setOptions(request.options);

      case ETextureSurfaceRequest.LIGHTING:
        return this.scene?.setLighting(request.lighting);

      case ETextureSurfaceRequest.FRAME_RATE:
        return this.scene?.setFrameRateLimit(request.limit);

      case ETextureSurfaceRequest.INPUT:
        return this.element?.dispatch(request.event);

      case ETextureSurfaceRequest.DRAG_LIGHT: {
        const lighting = this.scene?.dragLight(request.deltaX, request.deltaY);

        return lighting && this.reply({ kind: ETextureSurfaceResponse.LIGHTING, lighting });
      }

      case ETextureSurfaceRequest.DOLLY:
        return this.scene?.dolly(request.step);

      case ETextureSurfaceRequest.RESET:
        return this.scene?.reset();

      case ETextureSurfaceRequest.DISPOSE:
        return this.dispose();
    }
  }

  private start(request: Extract<TTextureSurfaceRequest, { kind: ETextureSurfaceRequest.START }>): void {
    this.dispose();

    this.log.info("Drawing on a canvas of", request.width, "x", request.height, "at", request.pixelRatio);

    this.target = new OffscreenRenderTarget(request.canvas, request);
    this.element = new RenderProxyElement(request, (cursor: string) =>
      this.reply({ cursor, kind: ETextureSurfaceResponse.CURSOR })
    );
    this.scene = new TextureSurfaceScene(this.target, this.element);

    this.scene.setReporter((cost) => this.reply({ cost, kind: ETextureSurfaceResponse.REPORT }));
  }

  private dispose(): void {
    if (this.scene) {
      this.log.info("Releasing everything the surface held");
    }

    this.scene?.dispose();
    this.scene = null;
    this.target = null;
    this.element = null;
  }
}
