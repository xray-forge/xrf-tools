import { InjectionToken } from "@wirestate/core";
import { Nullable } from "@xrf/types";

import { IVisualModelViews } from "@/core/visuals/lib/visual-views";

/** How a model stands: a frame of one of its motions, which the renderer bakes and poses it by itself. */
export interface IVisualPose {
  /** The motion playing, by its name, or null for the bind pose. */
  motion: Nullable<string>;
  /** Which of its frames the model is posed by. */
  frame: number;
}

/** A model standing as it was authored, which is what a surface that plays nothing shows. */
export const BIND_POSE: IVisualPose = { frame: 0, motion: null };

/** Nothing collapsed, for a surface that offers no way to collapse anything. */
export const NO_HIDDEN_BONES: ReadonlySet<number> = new Set();

/**
 * The model a viewport draws, whichever application put it there.
 */
export interface IVisualRenderSource {
  /** The open model's session, which the renderer reads it through, or null when nothing is open. */
  sessionId: Nullable<string>;
  /** What is open, or null when nothing is. */
  model: Nullable<IVisualModelViews>;
  /** Whether any submesh binds a bump pair, which is what makes the bump toggle mean something. */
  hasBump: boolean;
  /** How the model stands, absent on a surface that plays nothing. */
  pose?: IVisualPose;
  /** Joint to mark, already resolved to a position, absent on a surface that marks none. */
  highlightedJoint?: Nullable<[number, number, number]>;
  /** Bones to collapse, by index, already including the descendants each one hides. */
  hiddenBoneIndices?: ReadonlySet<number>;
}

/**
 * The model this application's viewport draws.
 */
export const VISUAL_RENDER_SOURCE: InjectionToken<IVisualRenderSource> = new InjectionToken<IVisualRenderSource>(
  "VISUAL_RENDER_SOURCE"
);
