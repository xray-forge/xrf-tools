import { Nullable } from "@xrf/types";

/** Floats one bone's transform takes: a 3x4, the rotation columns then the translation. */
export const RENDERER_FLOATS_PER_BONE: number = 12;

/**
 * A skeleton skinned objects bind to, in its bind pose.
 */
export interface IRendererSkeleton {
  /** Every bone's bind transform in model space, {@link RENDERER_FLOATS_PER_BONE} floats each. */
  binds: Float32Array;
  /** Child and parent bone indices, two a segment, for the skeleton overlay; left out, it draws nothing. */
  pairs?: Uint16Array;
}

/**
 * A motion baked to model space bone transforms, every frame.
 */
export interface IRendererMotion {
  /** Frame major: each frame is every bone's transform in turn. */
  transforms: Float32Array;
  /** Floats one bone takes in `transforms`, the bind layout's twelve at least. */
  floatsPerBone: number;
}

/**
 * How a skeleton stands now.
 */
export interface IRendererPose {
  /** The key the motion was put under, or null for the bind pose. */
  motion: Nullable<string>;
  frame: number;
  /** Bones collapsed to nothing, descendants included, as the engine hides a part that is not attached. */
  hiddenBones: ReadonlyArray<number>;
}
