import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { IVisualPose } from "@/core/visuals/lib/render/visual-render-source";
import { IVisualRenderer, IVisualRendererEvents } from "@/core/visuals/lib/render/visual-renderer";
import { IVisualPreviewViewOptions, VisualPreviewScene } from "@/core/visuals/lib/scene";
import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { Nullable } from "@/lib/types/general";

/** What a renderer needs to exist at all: somewhere to draw, and somewhere to report to. */
export interface IVisualLocalRendererOptions {
  target: DomRenderTarget;
  events: IVisualRendererEvents;
}

/**
 * Draws the model on the thread that asked.
 */
export class VisualLocalRenderer implements IVisualRenderer {
  private readonly scene: VisualPreviewScene;

  public constructor({ target, events }: IVisualLocalRendererOptions) {
    // On the canvas rather than the element around it: it is what the controls listen to and capture on.
    this.scene = new VisualPreviewScene(target, target.canvas, null);

    this.scene.setReporter((cost) => events.onReport(cost));
  }

  public setModel(model: Nullable<IVisualModelViews>): void {
    this.scene.setModel(model);
  }

  public applyTexture(submeshIndex: number, file: IVisualTextureFile): void {
    this.scene.applyTexture(submeshIndex, file);
  }

  public applyBump(submeshIndex: number, files: IVisualBumpFiles): void {
    this.scene.applyBump(submeshIndex, files);
  }

  public setPose(pose: IVisualPose): void {
    this.scene.setPose(pose.transforms, pose.frame, pose.floatsPerBone);
  }

  public setHiddenBones(bones: ReadonlySet<number>): void {
    this.scene.setHiddenBones(bones);
  }

  public setHighlightedJoint(position: Nullable<[number, number, number]>): void {
    this.scene.setHighlightedJoint(position);
  }

  public setDetailLevel(detail: number): void {
    this.scene.setDetailLevel(detail);
  }

  public applyViewOptions(options: IVisualPreviewViewOptions): void {
    this.scene.applyViewOptions(options);
  }

  public setLighting(lighting: IRenderLighting): void {
    this.scene.setLighting(lighting);
  }

  public setFrameRateLimit(limit: TFrameRateLimit): void {
    this.scene.setFrameRateLimit(limit);
  }

  public dolly(step: number): void {
    this.scene.dolly(step);
  }

  public resetCamera(): void {
    this.scene.resetCamera();
  }

  public dispose(): void {
    this.scene.setReporter(null);
    this.scene.dispose();
  }
}
