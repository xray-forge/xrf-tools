import { Maybe } from "@xrf/types";
import { BundleGroup, Object3D } from "three/webgpu";

import { TStaticBatchMesh } from "#/scene/static/static-batch";

/**
 * Batches one set of bundles holds: every bundle costs the frame on its own to replay, and a change records its
 * whole bundle again, so few enough to record in well under a millisecond and many enough to keep the bundles few.
 */
const BATCHES_PER_CHUNK: number = 32;

/** A bundle a phase, and how many batches stand in them. */
interface IBundleChunk {
  bundles: ReadonlyArray<BundleGroup>;
  count: number;
}

/** What a chunk holds of one batch: its meshes, a phase each. */
interface IBundleOwner {
  chunk: IBundleChunk;
  meshes: ReadonlyArray<TStaticBatchMesh>;
}

/**
 * The bundles a set of batches' meshes are recorded in, a chunk of batches to a bundle a phase, each phase's bundle in
 * that phase's scene: a surface's batches in the scene the first view draws and the one drawn after the second cull,
 * a shadow material's in each shadow view's.
 */
export class StaticBundleChunks {
  private readonly scenes: ReadonlyArray<Object3D>;
  private readonly chunks: Array<IBundleChunk> = [];
  private readonly owners: Map<object, IBundleOwner> = new Map();
  /** Whether its bundles draw at all, which a chunk made later takes too. */
  private isShown: boolean = true;

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
   * @param owner - What the meshes are of, which detaches them.
   * @param meshes - Its meshes, a phase each, recorded from now on in the first chunk with room.
   */
  public attach(owner: object, meshes: ReadonlyArray<TStaticBatchMesh>): void {
    this.detach(owner);

    let chunk: Maybe<IBundleChunk> = this.chunks.find((it: IBundleChunk) => it.count < BATCHES_PER_CHUNK);

    if (!chunk) {
      chunk = { bundles: this.scenes.map(() => StaticBundleChunks.createBundle(this.isShown)), count: 0 };
      this.chunks.push(chunk);
    }

    if (!chunk.count) {
      chunk.bundles.forEach((bundle: BundleGroup, phase: number) => this.scenes[phase].add(bundle));
    }

    chunk.bundles.forEach((bundle: BundleGroup, phase: number) => {
      bundle.add(meshes[phase]);
      bundle.needsUpdate = true;
    });
    chunk.count += 1;
    this.owners.set(owner, { chunk, meshes });
  }

  /**
   * @param owner - What meshes attached before are of, no longer recorded.
   */
  public detach(owner: object): void {
    const recorded: Maybe<IBundleOwner> = this.owners.get(owner);

    if (!recorded) {
      return;
    }

    const { chunk, meshes } = recorded;

    meshes.forEach((mesh: TStaticBatchMesh) => mesh.removeFromParent());
    this.owners.delete(owner);
    chunk.count -= 1;
    chunk.bundles.forEach((bundle: BundleGroup) => {
      bundle.needsUpdate = true;

      if (!chunk.count) {
        bundle.removeFromParent();
      }
    });
  }

  /**
   * @param isShown - Whether any of its bundles draws, every phase of them, now and once made.
   */
  public setShown(isShown: boolean): void {
    this.isShown = isShown;
    this.chunks.forEach((chunk: IBundleChunk) =>
      chunk.bundles.forEach((bundle: BundleGroup) => (bundle.visible = isShown))
    );
  }

  public clear(): void {
    this.chunks.forEach((chunk: IBundleChunk) =>
      chunk.bundles.forEach((bundle: BundleGroup) => bundle.removeFromParent())
    );
    this.chunks.length = 0;
    this.owners.clear();
  }

  /**
   * A bundle standing where the scene stands: its batches' meshes are placed by the buffers, never by their matrices,
   * so neither it nor anything in it takes part in three's walk of the scene's matrices.
   */
  private static createBundle(isShown: boolean): BundleGroup {
    const bundle: BundleGroup = new BundleGroup();

    bundle.visible = isShown;
    bundle.matrixAutoUpdate = false;
    bundle.matrixWorldAutoUpdate = false;

    return bundle;
  }
}
