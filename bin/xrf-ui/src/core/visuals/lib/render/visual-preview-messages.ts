import { IRenderFrameCost, TFrameRateLimit } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { IVisualPose } from "@/core/visuals/lib/render/visual-render-source";
import { IVisualPreviewViewOptions } from "@/core/visuals/lib/scene";
import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";

/**
 * What a model preview on another thread can be told about what it draws.
 */
export enum EVisualPreviewRequest {
  /** Draw this model, or none. */
  MODEL = "model",
  /** Draw this submesh with this file. */
  TEXTURE = "texture",
  /** Shade this submesh with this pair. */
  BUMP = "bump",
  /** Stand the model like this. */
  POSE = "pose",
  /** Collapse these bones. */
  HIDDEN_BONES = "hiddenBones",
  /** Mark this joint, or none. */
  JOINT = "joint",
  /** Draw every mesh this far down its collapse chain. */
  DETAIL = "detail",
  /** Draw it with these toggles. */
  OPTIONS = "options",
  /** Light it like this. */
  LIGHTING = "lighting",
  /** Redraw no more often than this. */
  FRAME_RATE = "frameRate",
  /** Move the camera towards the model or away from it. */
  DOLLY = "dolly",
  /** Back to the distance and angle the model is first framed from. */
  RESET = "reset",
}

/** What it says back about what it draws. */
export enum EVisualPreviewResponse {
  /** What frames are costing. */
  REPORT = "report",
}

/**
 * What a model preview on another thread is told, as messages.
 */
export type TVisualPreviewRequest =
  | { kind: EVisualPreviewRequest.MODEL; model: Nullable<IVisualModelViews> }
  | { kind: EVisualPreviewRequest.TEXTURE; submeshIndex: number; file: IVisualTextureFile }
  | { kind: EVisualPreviewRequest.BUMP; submeshIndex: number; files: IVisualBumpFiles }
  | { kind: EVisualPreviewRequest.POSE; pose: IVisualPose }
  | { kind: EVisualPreviewRequest.HIDDEN_BONES; bones: ReadonlySet<number> }
  | { kind: EVisualPreviewRequest.JOINT; position: Nullable<[number, number, number]> }
  | { kind: EVisualPreviewRequest.DETAIL; detail: number }
  | { kind: EVisualPreviewRequest.OPTIONS; options: IVisualPreviewViewOptions }
  | { kind: EVisualPreviewRequest.LIGHTING; lighting: IRenderLighting }
  | { kind: EVisualPreviewRequest.FRAME_RATE; limit: TFrameRateLimit }
  | { kind: EVisualPreviewRequest.DOLLY; step: number }
  | { kind: EVisualPreviewRequest.RESET };

/** What it says back, as messages. */
export type TVisualPreviewResponse = { kind: EVisualPreviewResponse.REPORT; cost: IRenderFrameCost };
