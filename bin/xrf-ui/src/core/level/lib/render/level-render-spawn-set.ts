import { RendererClient } from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { LevelSpawnCategory, LevelSpawnObject, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { ILevelSpawnDelivery, ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import {
  ILevelSpawnDressing,
  ILevelSpawnModelParts,
  toLevelSpawnModelParts,
  toLevelSpawnObject,
  toLevelSpawnSurface,
} from "@/core/level/lib/render/level-render-spawn";
import { LEVEL_SPAWN_CATEGORIES } from "@/core/level/lib/spawn/level-spawn-categories";

/** What of the renderer spawned objects are put through. */
export type TLevelRenderSpawnSink = Pick<
  RendererClient,
  "putGeometry" | "releaseGeometry" | "putObject" | "releaseObject" | "putSurface" | "releaseSurface"
>;

/** Every category shown, which is what a set shows until told otherwise. */
const EVERY_CATEGORY: ReadonlySet<LevelSpawnCategory> = new Set(LEVEL_SPAWN_CATEGORIES.map((it) => it.category));

/**
 * A level's spawned objects as the renderer holds them: each visual's geometry and surfaces put once, as its model
 * arrives, and one object for the objects of each category standing as it while that category is shown. All of it
 * goes with the objects it stands for.
 */
export class LevelRenderSpawnSet {
  private readonly sink: TLevelRenderSpawnSink;
  /** The objects the parts stand for, which a delivery of other objects replaces whole. */
  private objects: Nullable<LevelSpawnObjectsDescription> = null;
  /** The objects standing as each visual, by its index, then by category. */
  private standing: ReadonlyMap<number, ReadonlyMap<LevelSpawnCategory, ReadonlyArray<LevelSpawnObject>>> = new Map();
  /** Each object's hemisphere cube, by its index among the level's spawned objects. */
  private hemi: ReadonlyMap<number, ReadonlyArray<number>> = new Map();
  /** The visuals put, each its surfaces' keys by slot. */
  private readonly models: Map<number, ReadonlyArray<string>> = new Map();
  /** The objects put, by key. */
  private readonly shown: Set<string> = new Set();
  private visible: ReadonlySet<LevelSpawnCategory> = EVERY_CATEGORY;

  public constructor(sink: TLevelRenderSpawnSink) {
    this.sink = sink;
  }

  /**
   * @param delivery - The level's spawned objects and the models read so far, or null for none.
   */
  public stand(delivery: Nullable<ILevelSpawnDelivery>): void {
    if (delivery?.objects !== this.objects) {
      this.release();
      this.objects = delivery?.objects ?? null;
      this.standing = delivery ? groupByVisual(delivery.objects.objects) : new Map();
    }

    if (!delivery) {
      return;
    }

    this.hemi = delivery.hemi;
    delivery.models.forEach((model: ILevelSpawnModel, visual: number) => this.putModel(visual, model));
    this.showWanted();
  }

  /**
   * @param visible - The categories shown from now on; the others' objects go, their models kept.
   */
  public show(visible: ReadonlySet<LevelSpawnCategory>): void {
    this.visible = visible;
    this.showWanted();
  }

  /**
   * @param visual - The visual the objects stand as.
   * @param category - Their category.
   * @param instance - Which of the renderer's places of their object, which stand in the order they are held.
   * @returns The object standing there, or null for none held so.
   */
  public find(visual: number, category: LevelSpawnCategory, instance: number): Nullable<LevelSpawnObject> {
    return this.standing.get(visual)?.get(category)?.[instance] ?? null;
  }

  /**
   * @param visual - A visual objects stand as, by its index among the objects' visuals.
   * @returns Its name as the spawn gives it, or null for none held.
   */
  public nameVisual(visual: number): Nullable<string> {
    return this.objects?.visuals[visual] ?? null;
  }

  /** Lets everything go. */
  public release(): void {
    this.shown.forEach((key: string) => this.sink.releaseObject(key));
    this.models.forEach((surfaces: ReadonlyArray<string>, visual: number) => {
      surfaces.forEach((key: string) => this.sink.releaseSurface(key));
      this.sink.releaseGeometry(LEVEL_RENDER_KEYS.spawnGeometry(visual));
    });

    this.shown.clear();
    this.models.clear();
    this.objects = null;
    this.standing = new Map();
    this.hemi = new Map();
  }

  private putModel(visual: number, model: ILevelSpawnModel): void {
    if (this.models.has(visual) || !this.standing.has(visual)) {
      return;
    }

    const parts: ILevelSpawnModelParts = toLevelSpawnModelParts(visual, model);
    const surfaces: Array<string> = parts.dressings.map((dressing: ILevelSpawnDressing, slot: number) => {
      const key: string = LEVEL_RENDER_KEYS.spawnSurface(visual, slot);

      this.sink.putSurface(key, toLevelSpawnSurface(dressing));

      return key;
    });

    this.sink.putGeometry(LEVEL_RENDER_KEYS.spawnGeometry(visual), parts.geometry);
    this.models.set(visual, surfaces);
  }

  /** Puts the objects of every shown category whose model is put, and lets the others' go. */
  private showWanted(): void {
    this.models.forEach((surfaces: ReadonlyArray<string>, visual: number) => {
      this.standing.get(visual)?.forEach((objects: ReadonlyArray<LevelSpawnObject>, category: LevelSpawnCategory) => {
        const key: string = LEVEL_RENDER_KEYS.spawnObject(visual, category);
        const isWanted: boolean = this.visible.has(category);

        if (isWanted && !this.shown.has(key)) {
          this.sink.putObject(
            key,
            toLevelSpawnObject({
              geometry: LEVEL_RENDER_KEYS.spawnGeometry(visual),
              hemi: this.hemi,
              objects,
              surfaces,
            })
          );
          this.shown.add(key);
        } else if (!isWanted && this.shown.has(key)) {
          this.sink.releaseObject(key);
          this.shown.delete(key);
        }
      });
    });
  }
}

/** The objects standing as each visual, by category, in the order the spawn keeps them. */
function groupByVisual(
  objects: ReadonlyArray<LevelSpawnObject>
): Map<number, Map<LevelSpawnCategory, Array<LevelSpawnObject>>> {
  const groups: Map<number, Map<LevelSpawnCategory, Array<LevelSpawnObject>>> = new Map();

  for (const object of objects) {
    const categories: Map<LevelSpawnCategory, Array<LevelSpawnObject>> = groups.get(object.visual) ?? new Map();
    const group: Maybe<Array<LevelSpawnObject>> = categories.get(object.category);

    if (group) {
      group.push(object);
    } else {
      categories.set(object.category, [object]);
    }

    groups.set(object.visual, categories);
  }

  return groups;
}
