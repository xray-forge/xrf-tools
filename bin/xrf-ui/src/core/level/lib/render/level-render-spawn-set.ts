import { RendererClient } from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { LevelSpawnObject, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { ILevelSpawnDelivery, ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import { ILevelSpawnPart, toLevelSpawnParts, toLevelSpawnSurface } from "@/core/level/lib/render/level-render-spawn";

/** What of the renderer spawned objects are put through. */
export type TLevelRenderSpawnSink = Pick<
  RendererClient,
  "putGeometry" | "releaseGeometry" | "putObject" | "releaseObject" | "putSurface" | "releaseSurface"
>;

/**
 * A level's spawned objects as the renderer holds them: each visual's parts put once, as its model arrives, and all of
 * them let go with the objects they stand for.
 */
export class LevelRenderSpawnSet {
  private readonly sink: TLevelRenderSpawnSink;
  /** The objects the parts stand for, which a delivery of other objects replaces whole. */
  private objects: Nullable<LevelSpawnObjectsDescription> = null;
  /** The objects standing as each visual, by its index. */
  private standing: ReadonlyMap<number, ReadonlyArray<LevelSpawnObject>> = new Map();
  /** The parts put, by the visual they draw. */
  private readonly parts: Map<number, ReadonlyArray<ILevelSpawnPart>> = new Map();

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

    delivery?.models.forEach((model: ILevelSpawnModel, visual: number) => this.put(visual, model));
  }

  /** Lets every part go. */
  public release(): void {
    this.parts.forEach((parts: ReadonlyArray<ILevelSpawnPart>) => {
      for (const { key } of parts) {
        this.sink.releaseObject(key);
        this.sink.releaseSurface(key);
        this.sink.releaseGeometry(key);
      }
    });

    this.parts.clear();
    this.objects = null;
    this.standing = new Map();
  }

  private put(visual: number, model: ILevelSpawnModel): void {
    if (this.parts.has(visual)) {
      return;
    }

    const parts: Array<ILevelSpawnPart> = toLevelSpawnParts(visual, model, this.standing.get(visual) ?? []);

    this.parts.set(visual, parts);

    for (const part of parts) {
      this.sink.putGeometry(part.key, part.geometry);
      this.sink.putSurface(part.key, toLevelSpawnSurface(part.dressing));
      this.sink.putObject(part.key, part.object);
    }
  }
}

/** The objects standing as each visual, in the order the spawn keeps them. */
function groupByVisual(objects: ReadonlyArray<LevelSpawnObject>): Map<number, Array<LevelSpawnObject>> {
  const groups: Map<number, Array<LevelSpawnObject>> = new Map();

  for (const object of objects) {
    const group: Maybe<Array<LevelSpawnObject>> = groups.get(object.visual);

    if (group) {
      group.push(object);
    } else {
      groups.set(object.visual, [object]);
    }
  }

  return groups;
}
