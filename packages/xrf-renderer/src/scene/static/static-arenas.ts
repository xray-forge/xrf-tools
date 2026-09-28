import { Maybe, Nullable } from "@xrf/types";

import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { StaticArena } from "#/scene/static/static-arena";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticRoom } from "#/scene/static/static-room";
import { StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** A geometry placed in an arena, and how many objects draw it from there. */
interface IPlacement {
  range: Nullable<IStaticRange>;
  users: number;
}

/**
 * The arenas static geometry is copied into, one a vertex layout. A geometry is placed while any object draws it
 * statically, and its room freed once none does. An arena stays once made, empty or not, until the static draws go:
 * an object resolved against it may compile over its buffers at any time, and a new arena of the layout would be new
 * programs for every material drawing it. A geometry whose layout no arena can store is drawn plainly.
 */
export class StaticArenas {
  private readonly arenas: Map<string, StaticArena> = new Map();
  private readonly placements: Map<SceneGeometry, IPlacement> = new Map();
  /** Each geometry's layout, which takes sorting its attributes to tell. */
  private readonly signatures: WeakMap<SceneGeometry, Nullable<string>> = new WeakMap();
  private readonly buffers: StaticDrawBuffers;
  private readonly toUpcoming: () => Iterable<SceneGeometry>;

  /**
   * @param buffers - What every static draw reads: the lists and ranges a clustered draw reads, the storage limit
   *   that caps an arena's buffers, and where the buffers they replace go.
   * @param toUpcoming - The geometries objects still waiting to draw will draw statically, which a growing arena
   *   makes room for at once.
   */
  public constructor(buffers: StaticDrawBuffers, toUpcoming: () => Iterable<SceneGeometry>) {
    this.buffers = buffers;
    this.toUpcoming = toUpcoming;
  }

  /** Bumped whenever an arena replaces its buffers, which whatever draws them binds once it records again. */
  public get generation(): number {
    let generation: number = 0;

    this.arenas.forEach((arena: StaticArena) => (generation += arena.generation));

    return generation;
  }

  /**
   * @param geometry - A geometry.
   * @returns The arena of its layout, made where there is none yet; null for a layout no arena stores.
   */
  public toArena(geometry: SceneGeometry): Nullable<StaticArena> {
    const signature: Nullable<string> = this.toSignature(geometry);

    if (signature === null) {
      return null;
    }

    let arena: Maybe<StaticArena> = this.arenas.get(signature);

    if (!arena) {
      arena = new StaticArena(
        geometry.buffer,
        this.buffers.listEntries,
        this.buffers.clusterRangeWords,
        this.buffers.retirement
      );
      this.arenas.set(signature, arena);
    }

    return arena;
  }

  /**
   * @param geometry - A geometry one more object draws statically.
   * @returns Where it sits, or null where its arena cannot hold it and it has to be drawn plainly.
   */
  public acquire(geometry: SceneGeometry): Nullable<IStaticRange> {
    let placement: Maybe<IPlacement> = this.placements.get(geometry);

    if (!placement) {
      const arena: Nullable<StaticArena> = this.toArena(geometry);
      const limit: number = this.buffers.storageLimit;

      placement = {
        range: arena
          ? arena.place(geometry.buffer, () => this.toComing(arena, geometry), {
              indices: Math.floor(limit / Uint32Array.BYTES_PER_ELEMENT),
              vertices: Math.floor(limit / (arena.stride * Uint32Array.BYTES_PER_ELEMENT)),
            })
          : null,
        users: 0,
      };
      this.placements.set(geometry, placement);
    }

    placement.users += 1;

    return placement.range;
  }

  /**
   * @param geometry - A geometry one object fewer draws statically.
   */
  public release(geometry: SceneGeometry): void {
    const placement: Maybe<IPlacement> = this.placements.get(geometry);

    if (!placement || --placement.users > 0) {
      return;
    }

    this.placements.delete(geometry);

    if (!placement.range) {
      return;
    }

    placement.range.arena.free(placement.range);
  }

  private toSignature(geometry: SceneGeometry): Nullable<string> {
    let signature: Maybe<Nullable<string>> = this.signatures.get(geometry);

    if (signature === undefined) {
      signature = StaticArena.toSignature(geometry.buffer);
      this.signatures.set(geometry, signature);
    }

    return signature;
  }

  /** The room every upcoming geometry of an arena's layout will take, besides the one being placed. */
  private toComing(arena: StaticArena, placing: SceneGeometry): IStaticRoom {
    const room: IStaticRoom = { indices: 0, vertices: 0 };

    for (const geometry of new Set(this.toUpcoming())) {
      if (geometry !== placing && !this.placements.has(geometry) && this.toSignature(geometry) === arena.signature) {
        const vertices: number = geometry.buffer.getAttribute("position").count;

        room.vertices += vertices;
        room.indices += geometry.buffer.index?.count ?? vertices;
      }
    }

    return room;
  }

  public dispose(): void {
    this.arenas.forEach((arena: StaticArena) => arena.dispose());
    this.arenas.clear();
    this.placements.clear();
  }
}
