import { OffscreenRenderTarget } from "@/core/render/lib/frame/offscreen-render-target";
import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import { IRenderWorkerScene, TRenderWorkerReply } from "@/core/render/lib/worker/render-worker-host";
import {
  EVisualPreviewRequest,
  EVisualPreviewResponse,
  TVisualPreviewRequest,
  TVisualPreviewResponse,
} from "@/core/visuals/lib/render/visual-preview-messages";
import { VisualPreviewScene } from "@/core/visuals/lib/scene/VisualPreviewScene";

/**
 * Draws a model preview for somebody else's thread.
 */
export class VisualPreviewServer implements IRenderWorkerScene<TVisualPreviewRequest> {
  private readonly scene: VisualPreviewScene;

  public constructor(
    target: OffscreenRenderTarget,
    element: RenderProxyElement,
    reply: TRenderWorkerReply<TVisualPreviewResponse>
  ) {
    this.scene = new VisualPreviewScene(target, element, null);

    this.scene.setReporter((cost) => reply({ cost, kind: EVisualPreviewResponse.REPORT }));
  }

  /**
   * Takes one message about what is drawn.
   *
   * @param request - What the page said.
   */
  public take(request: TVisualPreviewRequest): void {
    switch (request.kind) {
      case EVisualPreviewRequest.MODEL:
        return this.scene.setModel(request.model);

      case EVisualPreviewRequest.TEXTURE:
        return this.scene.applyTexture(request.submeshIndex, request.file);

      case EVisualPreviewRequest.BUMP:
        return this.scene.applyBump(request.submeshIndex, request.files);

      case EVisualPreviewRequest.POSE:
        return this.scene.setPose(request.pose.transforms, request.pose.frame, request.pose.floatsPerBone);

      case EVisualPreviewRequest.HIDDEN_BONES:
        return this.scene.setHiddenBones(request.bones);

      case EVisualPreviewRequest.JOINT:
        return this.scene.setHighlightedJoint(request.position);

      case EVisualPreviewRequest.DETAIL:
        return this.scene.setDetailLevel(request.detail);

      case EVisualPreviewRequest.OPTIONS:
        return this.scene.applyViewOptions(request.options);

      case EVisualPreviewRequest.LIGHTING:
        return this.scene.setLighting(request.lighting);

      case EVisualPreviewRequest.FRAME_RATE:
        return this.scene.setFrameRateLimit(request.limit);

      case EVisualPreviewRequest.DOLLY:
        return this.scene.dolly(request.step);

      case EVisualPreviewRequest.RESET:
        return this.scene.resetCamera();
    }
  }

  public dispose(): void {
    this.scene.setReporter(null);
    this.scene.dispose();
  }
}
