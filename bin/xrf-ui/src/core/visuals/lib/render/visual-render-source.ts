import { InjectionToken } from "@wirestate/core";

import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { Nullable } from "@/lib/types/general";

/** How a model stands this frame: baked bone transforms, and which frame of them to show. */
export interface IVisualPose {
  /** Baked transforms of whatever is playing, or null for the bind pose. */
  transforms: Nullable<Float32Array>;
  /** Which frame of those transforms the model is posed by. */
  frame: number;
  /** How many floats one bone occupies in them. */
  floatsPerBone: number;
}

/** A model standing as it was authored, which is what a surface that plays nothing shows. */
export const BIND_POSE: IVisualPose = { floatsPerBone: 0, frame: 0, transforms: null };

/** Nothing collapsed, for a surface that offers no way to collapse anything. */
export const NO_HIDDEN_BONES: ReadonlySet<number> = new Set();

/**
 * The model a viewport draws, whichever application put it there.
 */
export interface IVisualRenderSource {
  /** What is open, or null when nothing is. */
  model: Nullable<IVisualModelViews>;
  /** Texture files by submesh index, uploaded and applied by whichever side draws. */
  textures: ReadonlyMap<number, IVisualTextureFile>;
  /** Bump pair files by submesh index, shaded as they arrive. */
  bumps: ReadonlyMap<number, IVisualBumpFiles>;
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
