import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { IVisualPose } from "@/core/visuals/lib/render/visual-render-source";
import { IVisualPreviewViewOptions } from "@/core/visuals/lib/scene";
import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { Nullable } from "@/lib/types/general";

/** What a model preview says back, whichever thread it is drawn on. */
export interface IVisualRendererEvents {
  /** What frames are costing, a few times a second. */
  onReport(cost: IRenderFrameCost): void;
}

/**
 * Everything a model preview is told.
 */
export interface IVisualRenderer {
  /**
   * Takes the model to draw, or nothing at all.
   *
   * @param model - What the open produced, or null to clear the scene.
   */
  setModel(model: Nullable<IVisualModelViews>): void;
  /**
   * Draws one submesh with the file read for it.
   *
   * @param submeshIndex - Index the submesh reports.
   * @param file - The file, which the drawing side uploads.
   */
  applyTexture(submeshIndex: number, file: IVisualTextureFile): void;
  /**
   * Shades one submesh with the pair read for it.
   *
   * @param submeshIndex - Index the submesh reports.
   * @param files - Both halves, which the drawing side uploads.
   */
  applyBump(submeshIndex: number, files: IVisualBumpFiles): void;
  /**
   * Poses the model, or returns it to its bind pose.
   *
   * @param pose - Baked transforms and which frame of them to show.
   */
  setPose(pose: IVisualPose): void;
  /**
   * Collapses some of the model's bones.
   *
   * @param bones - Indices to collapse, already including their descendants.
   */
  setHiddenBones(bones: ReadonlySet<number>): void;
  /**
   * Points the joint marker at a place, or takes it off.
   *
   * @param position - Where to mark, in the scene's own coordinates, or null to mark nothing.
   */
  setHighlightedJoint(position: Nullable<[number, number, number]>): void;
  /**
   * Draws every mesh at a different point along its collapse chain.
   *
   * @param detail - How far down each chain to go, `0` being full detail.
   */
  setDetailLevel(detail: number): void;
  /**
   * Applies the toolbar's toggles.
   *
   * @param options - Wireframe, checkerboard, grid, axes, skeleton, bump and alpha.
   */
  applyViewOptions(options: IVisualPreviewViewOptions): void;
  /**
   * Takes what the model is lit with.
   *
   * @param lighting - The direction, its strength and colour, and the fill.
   */
  setLighting(lighting: IRenderLighting): void;
  /**
   * Caps how often the model is redrawn.
   *
   * @param limit - Frames a second to allow.
   */
  setFrameRateLimit(limit: TFrameRateLimit): void;
  /**
   * Moves the camera towards the model or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  dolly(step: number): void;
  /** Back to the distance and the angle the model is first framed from. */
  resetCamera(): void;
  /** Releases the renderer and everything it holds. */
  dispose(): void;
}
