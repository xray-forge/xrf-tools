/**
 * A geometry's clusters: runs of up to `RENDERER_CLUSTER_TRIANGLES` consecutive triangles of its index order, each
 * culled and drawn on its own. Every range a static draw is made of is cut into a run of them, the run's first cluster
 * starting at the range's first index and each next where the last ends.
 */
export interface IRendererClusters {
  /** Four unsigned integers a cluster: its first index, its triangles, the drawable it is cut from, then nothing. */
  ranges: Uint32Array;
  /** Four floats a cluster: the centre and radius of a sphere holding its vertices, in the geometry's own space. */
  spheres: Float32Array;
}

/** Triangles a cluster holds at most, which is what one cluster's draw takes. */
export const RENDERER_CLUSTER_TRIANGLES: number = 128;
