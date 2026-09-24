/**
 * What a static cull kept and what occlusion hid, for the frame report: three counts none of it, since it draws from
 * recorded bundles.
 */
export interface IStaticCullCounts {
  draws: number;
  triangles: number;
  /** Single draws in view that both phases found hidden. */
  occludedDraws: number;
  /** Places of instanced draws in view that both phases found hidden. */
  occludedInstances: number;
  /** Triangles of both. */
  occludedTriangles: number;
}
