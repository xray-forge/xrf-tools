import { uniform, uniformArray } from "three/tsl";
import { Vector2, Vector4 } from "three/webgpu";

import { OcclusionView } from "#/uniforms/occlusion-view";

/** Levels a depth pyramid holds at most: a quarter of the last each, which covers any drawing size in six or seven. */
export const OCCLUSION_PYRAMID_LEVELS: number = 8;

/**
 * What occlusion tests read: the view the depth pyramid was last built from, the view being drawn, and the pyramid's
 * layout, a level each: where it starts in its buffer, its width and height.
 */
export class OcclusionUniforms {
  /** The view the pyramid holds the depth of when the frame's first cull reads it: the last frame's. */
  public readonly previous: OcclusionView = new OcclusionView();
  /** The view being drawn, which the pyramid holds the depth of by the frame's second cull. */
  public readonly current: OcclusionView = new OcclusionView();
  public readonly levels: Array<Vector4> = Array.from({ length: OCCLUSION_PYRAMID_LEVELS }, () => new Vector4());
  public readonly levelNodes = uniformArray(this.levels, "vec4");
  /** How many levels the pyramid has, and the drawing size its first level reduces. */
  public readonly levelCount = uniform(0);
  public readonly size = uniform(new Vector2(1, 1));
}
