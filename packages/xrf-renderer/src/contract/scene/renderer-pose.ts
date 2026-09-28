import { Nullable } from "@xrf/types";

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
