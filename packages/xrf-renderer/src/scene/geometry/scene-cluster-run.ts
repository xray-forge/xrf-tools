/**
 * A run of a geometry's clusters, which together are one range of its indices.
 */
export interface ISceneClusterRun {
  /** The first cluster, by its position in the geometry's table. */
  start: number;
  /** Clusters in the run. */
  count: number;
}
