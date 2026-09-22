import { OffscreenRenderTarget } from "@xrf/renderer";

import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import { IRenderWorkerScene, TRenderWorkerReply } from "@/core/render/lib/worker/render-worker-host";
import {
  ETextureSurfaceRequest,
  ETextureSurfaceResponse,
  TTextureSurfaceRequest,
  TTextureSurfaceResponse,
} from "@/core/textures/lib/render/texture-surface-messages";
import { TextureSurfaceScene } from "@/core/textures/lib/scene/TextureSurfaceScene";

/**
 * Draws a lit surface for somebody else's thread.
 *
 * Only the surface's own vocabulary: the canvas it draws on and the gestures on it are the host's.
 */
export class TextureSurfaceServer implements IRenderWorkerScene<TTextureSurfaceRequest> {
  private readonly scene: TextureSurfaceScene;
  private readonly reply: TRenderWorkerReply<TTextureSurfaceResponse>;

  public constructor(
    target: OffscreenRenderTarget,
    element: RenderProxyElement,
    reply: TRenderWorkerReply<TTextureSurfaceResponse>
  ) {
    this.reply = reply;
    this.scene = new TextureSurfaceScene(target, element);

    this.scene.setReporter((cost) => this.reply({ cost, kind: ETextureSurfaceResponse.REPORT }));
  }

  /**
   * Takes one message about what is drawn.
   *
   * @param request - What the page said.
   */
  public take(request: TTextureSurfaceRequest): void {
    switch (request.kind) {
      case ETextureSurfaceRequest.TEXTURES:
        return this.scene.setTextures(request.files);

      case ETextureSurfaceRequest.OPTIONS:
        return this.scene.setOptions(request.options);

      case ETextureSurfaceRequest.LIGHTING:
        return this.scene.setLighting(request.lighting);

      case ETextureSurfaceRequest.FRAME_RATE:
        return this.scene.setFrameRateLimit(request.limit);

      case ETextureSurfaceRequest.DOLLY:
        return this.scene.dolly(request.step);

      case ETextureSurfaceRequest.RESET:
        return this.scene.reset();

      case ETextureSurfaceRequest.DRAG_LIGHT:
        // The gesture is gathered where the pointer is and applied where the light is, so where it put it has
        // to come back for the viewer to keep.
        return this.reply({
          kind: ETextureSurfaceResponse.LIGHTING,
          lighting: this.scene.dragLight(request.deltaX, request.deltaY),
        });
    }
  }

  public dispose(): void {
    this.scene.setReporter(null);
    this.scene.dispose();
  }
}
