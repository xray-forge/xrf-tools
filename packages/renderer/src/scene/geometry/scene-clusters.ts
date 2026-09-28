import { Maybe, Nullable } from "@xrf/types";

import { IRendererClusters } from "#/contract/scene/renderer-clusters";
import { ISceneClusterRun } from "#/scene/geometry/scene-cluster-run";

/** Unsigned integers one cluster's range takes, and floats its sphere. */
export const SCENE_CLUSTER_WORDS: number = 4;

/**
 * A geometry's clusters as they crossed, and the runs its ranges are cut into, found from the table itself: a range's
 * run starts at the first cluster at its first index and goes on through the clusters that each start where the last
 * ends, to its end.
 */
export class SceneClusters {
  /** Four words a cluster: first index, triangles, drawable, nothing. */
  public readonly ranges: Uint32Array;
  /** Four floats a cluster: its sphere in the geometry's own space. */
  public readonly spheres: Float32Array;
  /** The first cluster starting at each index some cluster starts at. */
  private readonly firsts: Map<number, number> = new Map();

  public constructor(clusters: IRendererClusters) {
    this.ranges = clusters.ranges;
    this.spheres = clusters.spheres;

    for (let cluster = this.count - 1; cluster >= 0; cluster -= 1) {
      this.firsts.set(this.ranges[cluster * SCENE_CLUSTER_WORDS], cluster);
    }
  }

  /** Clusters in the table. */
  public get count(): number {
    return this.ranges.length / SCENE_CLUSTER_WORDS;
  }

  /**
   * @param start - A range's first index.
   * @param count - Indices in it.
   * @returns The run it is cut into, or null where the table holds none that is exactly it.
   */
  public toRun(start: number, count: number): Nullable<ISceneClusterRun> {
    const first: Maybe<number> = this.firsts.get(start);

    if (first === undefined || count <= 0) {
      return null;
    }

    let at: number = start;
    let cluster: number = first;

    while (cluster < this.count && at < start + count && this.ranges[cluster * SCENE_CLUSTER_WORDS] === at) {
      at += this.ranges[cluster * SCENE_CLUSTER_WORDS + 1] * 3;
      cluster += 1;
    }

    return at === start + count ? { count: cluster - first, start: first } : null;
  }
}
