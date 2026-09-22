import { OffscreenRenderTarget } from "@/core/render/lib/frame/offscreen-render-target";
import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import {
  EVisualPreviewRequest,
  EVisualPreviewResponse,
  TVisualPreviewRequest,
  TVisualPreviewResponse,
} from "@/core/visuals/lib/render/visual-preview-messages";
import { VisualPreviewScene } from "@/core/visuals/lib/scene/VisualPreviewScene";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** Where a served preview sends what it has to say. */
export type TVisualPreviewReply = (response: TVisualPreviewResponse) => void;

/**
 * Draws a model preview for somebody else's thread.
 */
export class VisualPreviewServer {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly reply: TVisualPreviewReply;

  private scene: Nullable<VisualPreviewScene> = null;
  private target: Nullable<OffscreenRenderTarget> = null;
  private element: Nullable<RenderProxyElement> = null;

  public constructor(reply: TVisualPreviewReply) {
    this.reply = reply;
  }

  /**
   * Takes one message.
   *
   * @param request - What the page said.
   */
  public take(request: TVisualPreviewRequest): void {
    switch (request.kind) {
      case EVisualPreviewRequest.START:
        return this.start(request);

      case EVisualPreviewRequest.RESIZE:
        this.target?.resize(request);
        this.element?.resize(request);

        return;

      case EVisualPreviewRequest.MODEL:
        return this.scene?.setModel(request.model);

      case EVisualPreviewRequest.TEXTURE:
        return this.scene?.applyTexture(request.submeshIndex, request.file);

      case EVisualPreviewRequest.BUMP:
        return this.scene?.applyBump(request.submeshIndex, request.files);

      case EVisualPreviewRequest.POSE:
        return this.scene?.setPose(request.pose.transforms, request.pose.frame, request.pose.floatsPerBone);

      case EVisualPreviewRequest.HIDDEN_BONES:
        return this.scene?.setHiddenBones(request.bones);

      case EVisualPreviewRequest.JOINT:
        return this.scene?.setHighlightedJoint(request.position);

      case EVisualPreviewRequest.DETAIL:
        return this.scene?.setDetailLevel(request.detail);

      case EVisualPreviewRequest.OPTIONS:
        return this.scene?.applyViewOptions(request.options);

      case EVisualPreviewRequest.LIGHTING:
        return this.scene?.setLighting(request.lighting);

      case EVisualPreviewRequest.FRAME_RATE:
        return this.scene?.setFrameRateLimit(request.limit);

      case EVisualPreviewRequest.INPUT:
        return this.element?.dispatch(request.event);

      case EVisualPreviewRequest.DOLLY:
        return this.scene?.dolly(request.step);

      case EVisualPreviewRequest.RESET:
        return this.scene?.resetCamera();

      case EVisualPreviewRequest.DISPOSE:
        return this.dispose();
    }
  }

  private start(request: Extract<TVisualPreviewRequest, { kind: EVisualPreviewRequest.START }>): void {
    this.dispose();

    this.log.info("Drawing on a canvas of", request.width, "x", request.height, "at", request.pixelRatio);

    this.target = new OffscreenRenderTarget(request.canvas, request);
    this.element = new RenderProxyElement(request, (cursor: string) =>
      this.reply({ cursor, kind: EVisualPreviewResponse.CURSOR })
    );
    this.scene = new VisualPreviewScene(this.target, this.element, null);

    this.scene.setReporter((cost) => this.reply({ cost, kind: EVisualPreviewResponse.REPORT }));
  }

  private dispose(): void {
    if (this.scene) {
      this.log.info("Releasing everything the preview held");
    }

    this.scene?.dispose();
    this.scene = null;
    this.target = null;
    this.element = null;
  }
}
