import { Maybe } from "@xrf/types";
import { BundleGroup, Object3D } from "three/webgpu";

import type { StaticBatch } from "#/scene/static/static-batch";

/**
 * Batches one pair of bundles holds: every bundle costs the frame on its own to replay, and a change records its
 * whole bundle again, so few enough to record in well under a millisecond and many enough to keep the bundles few.
 */
const BATCHES_PER_CHUNK: number = 32;

/** A pair of bundles, one a phase, and how many batches stand in them. */
interface IBundleChunk {
  early: BundleGroup;
  late: BundleGroup;
  count: number;
}

/**
 * The bundles the batches' meshes are recorded in, a chunk of batches to a pair: the early bundle in the scene the
 * first draw stands in, the late one in the scene drawn after the second cull.
 */
export class StaticBundleChunks {
  private readonly scene: Object3D;
  private readonly late: Object3D;
  private readonly chunks: Array<IBundleChunk> = [];
  private readonly owners: Map<StaticBatch, IBundleChunk> = new Map();

  /**
   * @param scene - Where the early bundles stand.
   * @param late - Where the late ones stand.
   */
  public constructor(scene: Object3D, late: Object3D) {
    this.scene = scene;
    this.late = late;
  }

  /** How many bundles stand in the scenes, early and late together. */
  public get bundles(): number {
    return this.chunks.filter((chunk: IBundleChunk) => chunk.count > 0).length * 2;
  }

  /**
   * @param batch - A batch whose meshes are recorded from now on, in the first chunk with room.
   */
  public attach(batch: StaticBatch): void {
    let chunk: Maybe<IBundleChunk> = this.chunks.find((it: IBundleChunk) => it.count < BATCHES_PER_CHUNK);

    if (!chunk) {
      chunk = { count: 0, early: new BundleGroup(), late: new BundleGroup() };
      this.chunks.push(chunk);
    }

    if (!chunk.count) {
      this.scene.add(chunk.early);
      this.late.add(chunk.late);
    }

    const [early, late] = batch.meshes;

    chunk.early.add(early);
    chunk.late.add(late);
    chunk.count += 1;
    chunk.early.needsUpdate = true;
    chunk.late.needsUpdate = true;
    this.owners.set(batch, chunk);
  }

  /**
   * @param batch - A batch whose meshes are no longer recorded.
   */
  public detach(batch: StaticBatch): void {
    const chunk: Maybe<IBundleChunk> = this.owners.get(batch);

    if (!chunk) {
      return;
    }

    batch.meshes.forEach((mesh) => mesh.removeFromParent());
    this.owners.delete(batch);
    chunk.count -= 1;
    chunk.early.needsUpdate = true;
    chunk.late.needsUpdate = true;

    if (!chunk.count) {
      chunk.early.removeFromParent();
      chunk.late.removeFromParent();
    }
  }

  public clear(): void {
    this.chunks.forEach((chunk: IBundleChunk) => {
      chunk.early.removeFromParent();
      chunk.late.removeFromParent();
    });
    this.chunks.length = 0;
    this.owners.clear();
  }
}
