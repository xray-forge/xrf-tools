/**
 * What a static cull kept and what occlusion hid, for the frame report: three counts none of it, since it draws from
 * recorded bundles.
 */
export interface IStaticCullCounts {
  /** Clusters drawn, over both of the camera's views. */
  clusters: number;
  triangles: number;
  /** Clusters in view that both phases found hidden. */
  occludedClusters: number;
  /** Their triangles. */
  occludedTriangles: number;
}
