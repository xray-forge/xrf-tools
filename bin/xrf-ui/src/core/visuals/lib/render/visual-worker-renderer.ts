import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { RenderWorkerClient } from "@/core/render/lib/worker/render-worker-client";
import {
  EVisualPreviewRequest,
  TVisualPreviewRequest,
  TVisualPreviewResponse,
} from "@/core/visuals/lib/render/visual-preview-messages";
import { IVisualPose } from "@/core/visuals/lib/render/visual-render-source";
import { IVisualRenderer, IVisualRendererEvents } from "@/core/visuals/lib/render/visual-renderer";
import { IVisualPreviewViewOptions } from "@/core/visuals/lib/scene";
import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { Logger } from "@/lib/logging";
import { Nullable } from "@/lib/types/general";

/** What a renderer on another thread needs: somewhere to draw, and somewhere to report to. */
export interface IVisualWorkerRendererOptions {
  target: DomRenderTarget;
  events: IVisualRendererEvents;
}

/**
 * Draws the model on a thread of its own.
 */
export class VisualWorkerRenderer implements IVisualRenderer {
  public readonly log: Logger = new Logger(__MODULE_NAME__);

  private readonly client: RenderWorkerClient<TVisualPreviewRequest, TVisualPreviewResponse>;

  public constructor({ target, events }: IVisualWorkerRendererOptions) {
    this.client = new RenderWorkerClient({
      log: this.log,
      onResponse: (response: TVisualPreviewResponse): void => events.onReport(response.cost),
      target,
      worker: new Worker(new URL("./visual-preview.worker.ts", import.meta.url), { type: "module" }),
    });
  }

  public setModel(model: Nullable<IVisualModelViews>): void {
    this.client.post({ kind: EVisualPreviewRequest.MODEL, model });
  }

  public applyTexture(submeshIndex: number, file: IVisualTextureFile): void {
    this.client.post({ file, kind: EVisualPreviewRequest.TEXTURE, submeshIndex });
  }

  public applyBump(submeshIndex: number, files: IVisualBumpFiles): void {
    this.client.post({ files, kind: EVisualPreviewRequest.BUMP, submeshIndex });
  }

  public setPose(pose: IVisualPose): void {
    this.client.post({ kind: EVisualPreviewRequest.POSE, pose });
  }

  public setHiddenBones(bones: ReadonlySet<number>): void {
    this.client.post({ bones, kind: EVisualPreviewRequest.HIDDEN_BONES });
  }

  public setHighlightedJoint(position: Nullable<[number, number, number]>): void {
    this.client.post({ kind: EVisualPreviewRequest.JOINT, position });
  }

  public setDetailLevel(detail: number): void {
    this.client.post({ detail, kind: EVisualPreviewRequest.DETAIL });
  }

  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.client.post({ kind: EVisualPreviewRequest.OPTIONS, options });
  }

  public setLighting(lighting: IRenderLighting): void {
    this.client.post({ kind: EVisualPreviewRequest.LIGHTING, lighting });
  }

  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.client.post({ kind: EVisualPreviewRequest.FRAME_RATE, limit });
  }

  public dolly(step: number): void {
    this.client.post({ kind: EVisualPreviewRequest.DOLLY, step });
  }

  public resetCamera(): void {
    this.client.post({ kind: EVisualPreviewRequest.RESET });
  }

  public dispose(): void {
    this.client.dispose();
  }
}
