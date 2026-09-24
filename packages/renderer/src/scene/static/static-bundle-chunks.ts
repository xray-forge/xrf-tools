import { Maybe } from "@xrf/types";
import { BundleGroup, Mesh, Object3D } from "three/webgpu";

import { StaticBatch } from "#/scene/static/static-batch";

/**
 * Batches one set of bundles holds: every bundle costs the frame on its own to replay, and a change records its
 * whole bundle again, so few enough to record in well under a millisecond and many enough to keep the bundles few.
 */
const BATCHES_PER_CHUNK: number = 32;

/** A bundle a phase, the cell its batches stand in, and how many batches stand in them. */
interface IBundleChunk {
  bundles: ReadonlyArray<BundleGroup>;
  cell: string;
  count: number;
}

/**
 * The bundles a set of batches' meshes are recorded in, a chunk of batches to a bundle a phase, each phase's bundle in
 * that phase's scene: a surface's batches in the scene the first draw stands in and the one drawn after the second
 * cull, a shadow material's in each cascade's.
 */
export class StaticBundleChunks {
  private readonly scenes: ReadonlyArray<Object3D>;
  private readonly chunks: Array<IBundleChunk> = [];
  /** Each cell's chunks, so a chunk holds batches of one cell alone and a cell's bundles show or hide together. */
  private readonly cells: Map<string, Array<IBundleChunk>> = new Map();
  private readonly owners: Map<StaticBatch, IBundleChunk> = new Map();

  /**
   * @param scenes - Where each phase's bundles stand, in the order a batch's meshes are.
   */
  public constructor(scenes: ReadonlyArray<Object3D>) {
    this.scenes = scenes;
  }

  /** How many bundles stand in the scenes. */
  public get bundles(): number {
    return this.chunks.filter((chunk: IBundleChunk) => chunk.count > 0).length * this.scenes.length;
  }

  /**
   * @param batch - A batch whose meshes are recorded from now on, in the first chunk of its cell with room.
   * @param cell - The cell it stands in, which its chunk shows and hides with.
   */
  public attach(batch: StaticBatch, cell: string = ""): void {
    let chunks: Maybe<Array<IBundleChunk>> = this.cells.get(cell);

    if (!chunks) {
      chunks = [];
      this.cells.set(cell, chunks);
    }

    let chunk: Maybe<IBundleChunk> = chunks.find((it: IBundleChunk) => it.count < BATCHES_PER_CHUNK);

    if (!chunk) {
      chunk = { bundles: this.scenes.map(() => StaticBundleChunks.createBundle()), cell, count: 0 };
      chunks.push(chunk);
      this.chunks.push(chunk);
    }

    if (!chunk.count) {
      chunk.bundles.forEach((bundle: BundleGroup, phase: number) => this.scenes[phase].add(bundle));
    }

    chunk.bundles.forEach((bundle: BundleGroup, phase: number) => {
      bundle.add(batch.meshes[phase]);
      bundle.needsUpdate = true;
    });
    chunk.count += 1;
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

    batch.meshes.forEach((mesh: Mesh) => mesh.removeFromParent());
    this.owners.delete(batch);
    chunk.count -= 1;
    chunk.bundles.forEach((bundle: BundleGroup) => {
      bundle.needsUpdate = true;

      if (!chunk.count) {
        bundle.removeFromParent();
      }
    });
  }

  /**
   * Shows the bundles of one phase for the cells a view takes and hides the rest: three skips a hidden bundle whole,
   * so the draws of a cell the view does not reach are never issued. Nothing records again.
   *
   * @param phase - The phase, by its position among the scenes.
   * @param isShown - Whether the view takes a cell.
   */
  public show(phase: number, isShown: (cell: string) => boolean): void {
    for (const chunk of this.chunks) {
      chunk.bundles[phase].visible = isShown(chunk.cell);
    }
  }

  public clear(): void {
    this.chunks.forEach((chunk: IBundleChunk) =>
      chunk.bundles.forEach((bundle: BundleGroup) => bundle.removeFromParent())
    );
    this.chunks.length = 0;
    this.cells.clear();
    this.owners.clear();
  }

  /**
   * A bundle standing where the scene stands: its batches' meshes are placed by the buffers, never by their matrices,
   * so neither it nor anything in it takes part in three's walk of the scene's matrices.
   */
  private static createBundle(): BundleGroup {
    const bundle: BundleGroup = new BundleGroup();

    bundle.matrixAutoUpdate = false;
    bundle.matrixWorldAutoUpdate = false;

    return bundle;
  }
}
