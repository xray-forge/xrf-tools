/**
 * The pools of the static draw buffers, each grown on its own: slots a draw each, places an instance each, rows a
 * place of one instanced draw each, impostors of clumps of trees, clusters, batches, the lists a view's kept clusters
 * are written to, and the depth pyramid's texels.
 */
export enum EStaticPool {
  SLOTS = "slots",
  PLACES = "places",
  ROWS = "rows",
  LODS = "lods",
  CLUSTERS = "clusters",
  BATCHES = "batches",
  /** Entries of the camera's batches' regions, a view each of the camera's two. */
  SURFACE_LIST = "surfaceList",
  /** Entries of the shadow batches' regions, a view each of the shadow views. */
  SHADOW_LIST = "shadowList",
  PYRAMID = "pyramid",
}
