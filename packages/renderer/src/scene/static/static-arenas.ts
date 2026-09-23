import { Maybe, Nullable } from "@xrf/types";

import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { StaticArena } from "#/scene/static/static-arena";
import { IStaticRange } from "#/scene/static/static-range";
import { IStaticRoom } from "#/scene/static/static-room";

/** A geometry placed in an arena, and how many objects draw it from there. */
interface IPlacement {
  range: Nullable<IStaticRange>;
  users: number;
}

/**
 * The arenas static geometry is copied into, one a vertex layout. A geometry is placed while any object draws it
 * statically, and its room freed once none does; an arena holding nothing goes.
 */
export class StaticArenas {
  private readonly arenas: Map<string, StaticArena> = new Map();
  private readonly placements: Map<SceneGeometry, IPlacement> = new Map();
  /** Each geometry's layout, which takes sorting its attributes to tell. */
  private readonly signatures: WeakMap<SceneGeometry, string> = new WeakMap();
  private readonly onGrown: (arena: StaticArena) => void;
  private readonly onEmptied: (arena: StaticArena) => void;
  private readonly toUpcoming: () => Iterable<SceneGeometry>;

  /**
   * @param onGrown - Told an arena replaced its buffers, which whatever draws them has to draw instead.
   * @param onEmptied - Told an arena holds nothing and goes, with whatever draws it.
   * @param toUpcoming - The geometries objects still waiting to draw will draw statically, which a growing arena
   *   makes room for at once.
   */
  public constructor(
    onGrown: (arena: StaticArena) => void,
    onEmptied: (arena: StaticArena) => void,
    toUpcoming: () => Iterable<SceneGeometry>
  ) {
    this.onGrown = onGrown;
    this.onEmptied = onEmptied;
    this.toUpcoming = toUpcoming;
  }

  /**
   * @param geometry - A geometry.
   * @returns The arena of its layout, made where there is none yet.
   */
  public toArena(geometry: SceneGeometry): StaticArena {
    const signature: string = this.toSignature(geometry);
    let arena: Maybe<StaticArena> = this.arenas.get(signature);

    if (!arena) {
      arena = new StaticArena(geometry.buffer);
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
      const arena: StaticArena = this.toArena(geometry);
      const generation: number = arena.generation;

      placement = { range: arena.place(geometry.buffer, () => this.toComing(arena, geometry)), users: 0 };
      this.placements.set(geometry, placement);

      if (arena.generation !== generation) {
        this.onGrown(arena);
      }
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

    const { arena } = placement.range;

    arena.free(placement.range);

    if (arena.isEmpty) {
      this.arenas.delete(arena.signature);
      this.onEmptied(arena);
      arena.dispose();
    }
  }

  private toSignature(geometry: SceneGeometry): string {
    let signature: Maybe<string> = this.signatures.get(geometry);

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
    this.arenas.forEach((arena: StaticArena) => {
      this.onEmptied(arena);
      arena.dispose();
    });
    this.arenas.clear();
    this.placements.clear();
  }
}
