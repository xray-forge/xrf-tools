import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IOffscreenRenderSize } from "@/core/render/lib/frame/offscreen-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { IRenderInputEvent } from "@/core/render/lib/worker/render-input";
import { RenderInputForwarder } from "@/core/render/lib/worker/render-input-forwarder";
import {
  EVisualPreviewRequest,
  EVisualPreviewResponse,
  listVisualPreviewTransfers,
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

  private readonly worker: Worker;
  private readonly target: DomRenderTarget;
  private readonly events: IVisualRendererEvents;
  private readonly input: RenderInputForwarder;
  private readonly unobserve: () => void;

  public constructor({ target, events }: IVisualWorkerRendererOptions) {
    this.target = target;
    this.events = events;

    this.worker = new Worker(new URL("./visual-preview.worker.ts", import.meta.url), { type: "module" });
    this.worker.onmessage = (event: MessageEvent<TVisualPreviewResponse>): void => this.receive(event.data);

    // A worker that dies on its way up says nothing at all otherwise: the page keeps posting into a thread
    // that will never answer, and the only symptom is a viewport that never draws a frame.
    this.worker.onerror = (event: ErrorEvent): void =>
      this.log.error("The visual preview worker failed:", event.message);

    this.log.info("Started a visual preview worker");

    this.post({
      canvas: target.canvas.transferControlToOffscreen(),
      kind: EVisualPreviewRequest.START,
      ...this.getSize(),
    });

    this.input = new RenderInputForwarder(target.canvas, (event: IRenderInputEvent) =>
      this.post({ event, kind: EVisualPreviewRequest.INPUT })
    );
    this.unobserve = target.observe(() => this.post({ kind: EVisualPreviewRequest.RESIZE, ...this.getSize() }));
  }

  public setModel(model: Nullable<IVisualModelViews>): void {
    this.post({ kind: EVisualPreviewRequest.MODEL, model });
  }

  public applyTexture(submeshIndex: number, file: IVisualTextureFile): void {
    this.post({ file, kind: EVisualPreviewRequest.TEXTURE, submeshIndex });
  }

  public applyBump(submeshIndex: number, files: IVisualBumpFiles): void {
    this.post({ files, kind: EVisualPreviewRequest.BUMP, submeshIndex });
  }

  public setPose(pose: IVisualPose): void {
    this.post({ kind: EVisualPreviewRequest.POSE, pose });
  }

  public setHiddenBones(bones: ReadonlySet<number>): void {
    this.post({ bones, kind: EVisualPreviewRequest.HIDDEN_BONES });
  }

  public setHighlightedJoint(position: Nullable<[number, number, number]>): void {
    this.post({ kind: EVisualPreviewRequest.JOINT, position });
  }

  public setDetailLevel(detail: number): void {
    this.post({ detail, kind: EVisualPreviewRequest.DETAIL });
  }

  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.post({ kind: EVisualPreviewRequest.OPTIONS, options });
  }

  public setLighting(lighting: IRenderLighting): void {
    this.post({ kind: EVisualPreviewRequest.LIGHTING, lighting });
  }

  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.post({ kind: EVisualPreviewRequest.FRAME_RATE, limit });
  }

  public dolly(step: number): void {
    this.post({ kind: EVisualPreviewRequest.DOLLY, step });
  }

  public resetCamera(): void {
    this.post({ kind: EVisualPreviewRequest.RESET });
  }

  public dispose(): void {
    this.unobserve();
    this.input.dispose();

    // Terminating reclaims the thread whole, context and all; the message is what releases a server that is
    // not on a worker, and is said first so the two are released the same way.
    this.post({ kind: EVisualPreviewRequest.DISPOSE });
    this.worker.terminate();

    this.log.info("Terminated the visual preview worker");
  }

  private post(request: TVisualPreviewRequest): void {
    this.worker.postMessage(request, listVisualPreviewTransfers(request));
  }

  private receive(response: TVisualPreviewResponse): void {
    switch (response.kind) {
      case EVisualPreviewResponse.REPORT:
        return this.events.onReport(response.cost);

      case EVisualPreviewResponse.CURSOR:
        return this.input.setCursor(response.cursor);
    }
  }

  private getSize(): IOffscreenRenderSize {
    return { height: this.target.height, pixelRatio: this.target.pixelRatio, width: this.target.width };
  }
}
