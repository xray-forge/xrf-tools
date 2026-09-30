import { RIGID_TRANSFORM_FLOATS } from "@xrf/math";

/** Floats one bone's transform takes: a 3x4, the rotation columns then the translation. */
export const RENDERER_FLOATS_PER_BONE: number = RIGID_TRANSFORM_FLOATS;

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
 * What of a skeleton moves between threads: its arrays, never copied.
 *
 * @param skeleton - The skeleton about to be posted.
 * @returns Its buffers.
 */
export function listRendererSkeletonTransfers(skeleton: IRendererSkeleton): Array<Transferable> {
  return skeleton.pairs ? [skeleton.binds.buffer, skeleton.pairs.buffer] : [skeleton.binds.buffer];
}
