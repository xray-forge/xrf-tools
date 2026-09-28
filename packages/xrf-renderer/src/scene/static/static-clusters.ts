import { Nullable } from "@xrf/types";
import { Matrix4, Sphere, Vector3 } from "three/webgpu";

import { DirtySpan } from "#/scene/dirty-span";
import { ISceneClusterRun } from "#/scene/geometry/scene-cluster-run";
import { SCENE_CLUSTER_WORDS, SceneClusters } from "#/scene/geometry/scene-clusters";
import { RangeAllocator } from "#/scene/static/range-allocator";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticRunPool } from "#/scene/static/static-run-pool";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";

/** A sphere and a point, reused. */
const SPHERE: Sphere = new Sphere();
const CENTRE: Vector3 = new Vector3();

/**
 * The clusters every static draw is made of, each slot's in a run of its own: where in its arena each cluster's
 * indices and vertices start, how many triangles it has and which slot it is of, and the sphere it is culled by.
 * Handed out in runs, uploaded as one span a buffer.
 */
export class StaticClusters implements IStaticRunPool {
  public readonly kind: EStaticPool = EStaticPool.CLUSTERS;

  private readonly buffers: StaticDrawBuffers;
  private readonly allocator: RangeAllocator = new RangeAllocator();
  private readonly span: DirtySpan = new DirtySpan();
  private currentVersion: number = 0;

  public constructor(buffers: StaticDrawBuffers) {
    this.buffers = buffers;
    this.allocator.grow(buffers.capacity(EStaticPool.CLUSTERS));
  }

  public get capacity(): number {
    return this.allocator.capacity;
  }

  public get used(): number {
    return this.allocator.used;
  }

  /** Clusters the cull has to look at: up to the end of the last run handed out. */
  public get extent(): number {
    return this.allocator.extent;
  }

  /** Bumped whenever a cluster changes, so a cull knows to run again. */
  public get version(): number {
    return this.currentVersion;
  }

  public fits(count: number): boolean {
    return this.allocator.fits(count);
  }

  public allocate(count: number): Nullable<number> {
    return this.allocator.allocate(count);
  }

  public grow(capacity: number): void {
    this.buffers.grow(EStaticPool.CLUSTERS, capacity);
    this.allocator.grow(capacity);
    this.currentVersion += 1;
  }

  /**
   * Copies a run of a geometry's clusters in for one slot, onto where its geometry sits in its arena.
   *
   * @param start - Where the slot's run starts.
   * @param slot - The slot.
   * @param clusters - The geometry's clusters.
   * @param run - Which of them the slot draws.
   * @param range - Where the geometry sits in its arena.
   * @param placement - What stands a single draw's clusters where they draw, whose spheres are then in renderer space;
   *   null for an instanced draw's, which each of its places stands.
   */
  public write(
    start: number,
    slot: number,
    clusters: SceneClusters,
    run: ISceneClusterRun,
    range: IStaticRange,
    placement: Nullable<Matrix4>
  ): void {
    const ranges: Uint32Array = this.buffers.clusterRanges.array as Uint32Array;
    const spheres: Float32Array = this.buffers.clusterSpheres.array as Float32Array;

    for (let index = 0; index < run.count; index += 1) {
      const source: number = (run.start + index) * SCENE_CLUSTER_WORDS;
      const at: number = (start + index) * 4;

      ranges[at] = range.indexStart + clusters.ranges[source];
      ranges[at + 1] = clusters.ranges[source + 1];
      ranges[at + 2] = range.vertexStart;
      ranges[at + 3] = slot;

      if (placement) {
        SPHERE.set(
          CENTRE.set(clusters.spheres[source], clusters.spheres[source + 1], clusters.spheres[source + 2]),
          clusters.spheres[source + 3]
        ).applyMatrix4(placement);
        spheres.set([SPHERE.center.x, SPHERE.center.y, SPHERE.center.z, SPHERE.radius], at);
      } else {
        spheres.set(clusters.spheres.subarray(source, source + 4), at);
      }
    }

    this.span.touch(start, start + run.count - 1);
    this.currentVersion += 1;
  }

  /** Leaves a slot's run drawing nothing, free for another. */
  public free(start: number, count: number): void {
    const ranges: Uint32Array = this.buffers.clusterRanges.array as Uint32Array;

    // Of no triangles, a cluster left behind draws nothing if the cull still reaches it.
    for (let index = 0; index < count; index += 1) {
      ranges[(start + index) * 4 + 1] = 0;
    }

    this.allocator.release(start, count);
    this.span.touch(start, start + count - 1);
    this.currentVersion += 1;
  }

  public flush(): void {
    this.span.upload(this.buffers.clusterRanges, 4);
    this.span.upload(this.buffers.clusterSpheres, 4);
    this.span.clear();
  }
}
