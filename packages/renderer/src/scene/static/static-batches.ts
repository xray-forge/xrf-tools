import { Maybe, Nullable } from "@xrf/types";
import { Box3, Material, Object3D, Vector4 } from "three/webgpu";

import { ISurfaceMaterial } from "#/material/surface-material";
import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch, TStaticBatchArguments } from "#/scene/static/static-batch";
import { StaticBundleChunks } from "#/scene/static/static-bundle-chunks";
import { isBoxInPlanes, STATIC_EVERYWHERE, toStaticCell } from "#/scene/static/static-cell";
import { EStaticDrawKind } from "#/scene/static/static-draw-kind";
import { StaticDrawPool } from "#/scene/static/static-draw-pool";
import { StaticShadowChanges } from "#/scene/static/static-shadow-changes";

/** An arena's batches of one kind for one grouping: the ones drawing, by material and cell, and the idle ones kept. */
interface IArenaBatches {
  drawing: Map<string, StaticBatch>;
  idle: Array<StaticBatch>;
}

/**
 * One way of batching the slots: the arguments each batch's phases draw by, which material of a surface groups its
 * slots, and the bundles the batches are recorded in.
 */
interface IBatchGrouping {
  phases: ReadonlyArray<TStaticBatchArguments>;
  chunks: StaticBundleChunks;
  arenas: Map<StaticArena, Record<EStaticDrawKind, IArenaBatches>>;
  /** The batch drawing each slot. */
  drawing: Map<number, StaticBatch>;
  /** Each batch's key among its arena's, its material and cell. */
  keys: Map<StaticBatch, string>;
  /** Whether it batches by cell too, and the box of everything standing in each cell. */
  isCelled: boolean;
  cells: Map<string, Box3>;
  /** Whether its batches draw the arenas' line indices, as a wireframe. */
  isWire: boolean;
}

/**
 * The batches drawing every static draw, one an arena, kind and material, each recorded in a chunk's bundles while it
 * draws any. Every slot is batched twice over: by its surface's own material, drawn in the G-buffer, and, for a
 * surface that casts, by its shadow material and the cell it stands in, drawn into each cascade. Every opaque surface
 * shares one shadow material, so its casters are a batch an arena, kind and cell, however many surfaces they are; a
 * cascade shows only the cells its box reaches, so it issues the draws of what it can cast from alone. While a wireframe
 * draws, every slot is batched a third time, by one wireframe material over its arena's line index, in place of its
 * surface's batch.
 */
export class StaticBatches {
  private readonly surfaces: IBatchGrouping;
  private readonly shadows: IBatchGrouping;
  private readonly wires: IBatchGrouping;
  /** Each slot's surface, which its wireframe batch is chosen by. */
  private readonly slotSurfaces: Map<number, ISurfaceMaterial> = new Map();
  /** What a wireframe draws every surface with but an impostor, or null while none draws. */
  private wireMaterial: Nullable<Material> = null;
  private currentVersion: number = 0;
  /** Where what the shadow views draw changed, and what of it sways. */
  public readonly shadowChanges: StaticShadowChanges = new StaticShadowChanges();

  /**
   * @param pool - The slots the batches draw.
   * @param scene - Where a batch drawing anything stands.
   * @param late - Where its draw by the second cull's arguments stands.
   * @param cascadeScenes - Where a shadow batch's draws by each cascade's arguments stand.
   */
  public constructor(pool: StaticDrawPool, scene: Object3D, late: Object3D, cascadeScenes: ReadonlyArray<Object3D>) {
    this.surfaces = StaticBatches.createGrouping([() => pool.args, () => pool.lateArgs], [scene, late], false);
    this.wires = StaticBatches.createGrouping(
      [() => pool.wireArgs, () => pool.wireLateArgs],
      [scene, late],
      false,
      true
    );
    this.shadows = StaticBatches.createGrouping(
      cascadeScenes.map((_, view: number) => () => pool.viewArgs[view]),
      cascadeScenes,
      true
    );
  }

  /** Bumped whenever what any batch draws changed, so a shadow map drawn before is drawn again. */
  public get version(): number {
    return this.currentVersion;
  }

  /**
   * Shows a cascade the cells its box reaches, and hides the rest.
   *
   * @param view - The cascade.
   * @param planes - Its box's planes.
   */
  public showShadowCells(view: number, planes: ReadonlyArray<Vector4>): void {
    this.shadows.chunks.show(view, (cell: string) => {
      const box: Maybe<Box3> = this.shadows.cells.get(cell);

      return cell === STATIC_EVERYWHERE || (box !== undefined && isBoxInPlanes(box, planes));
    });
  }

  /** Every material a batch draws in the G-buffer. */
  public get materials(): Iterable<Material> {
    return StaticBatches.all(this.surfaces).flatMap((batches: IArenaBatches) =>
      [...batches.drawing.values()].map((batch: StaticBatch) => batch.material as Material)
    );
  }

  /**
   * @param slot - A slot drawing from now on in the batch of its arena and material, and in no other. Its batch
   *   records again even where it drew the slot already: only a recording uploads the matrices its draws read, since
   *   a replay refreshes nothing of the batch's own.
   * @param arena - The arena its geometry sits in.
   * @param kind - The kind of static draw it is.
   * @param surface - What draws it.
   * @param bounds - What it spans in renderer space: a single draw's sphere's box, or the box of every place an
   *   instanced one stands in; null where it is not known.
   */
  public put(
    slot: number,
    arena: StaticArena,
    kind: EStaticDrawKind,
    surface: ISurfaceMaterial,
    bounds: Nullable<Box3> = null
  ): void {
    StaticBatches.put(this.surfaces, slot, arena, kind, surface.material, surface.keys, bounds);
    this.slotSurfaces.set(slot, surface);

    if (this.wireMaterial) {
      this.putWire(slot, arena, kind, surface, this.wireMaterial);
    }

    if (surface.shadow) {
      StaticBatches.put(this.shadows, slot, arena, kind, surface.shadow, surface.shadowKeys, bounds);
    } else {
      StaticBatches.withdraw(this.shadows, slot);
    }

    this.shadowChanges.put(slot, bounds, Boolean(surface.shadow), arena.isSwaying);
    this.currentVersion += 1;
  }

  /**
   * @param slot - A slot no batch draws from now on.
   */
  public withdraw(slot: number): void {
    StaticBatches.withdraw(this.surfaces, slot);
    StaticBatches.withdraw(this.shadows, slot);
    StaticBatches.withdraw(this.wires, slot);
    this.slotSurfaces.delete(slot);
    this.shadowChanges.withdraw(slot);
    this.currentVersion += 1;
  }

  /**
   * Draws every slot as its triangles' edges, or as its triangles again: by one material over the arenas' line
   * indices, or by its surface's own. An impostor's slot keeps its own material, which turns its quad to the camera.
   *
   * @param material - What a wireframe draws with, or null to draw the triangles.
   */
  public setWireframe(material: Nullable<Material>): void {
    if (material === this.wireMaterial) {
      return;
    }

    this.wireMaterial = material;

    // Its batches go idle rather than away: their geometries share the arenas' buffers, which disposing would free.
    [...this.wires.drawing.keys()].forEach((slot: number) => StaticBatches.withdraw(this.wires, slot));

    if (material) {
      this.surfaces.drawing.forEach((batch: StaticBatch, slot: number) =>
        this.putWire(slot, batch.arena, batch.kind, this.slotSurfaces.get(slot) as ISurfaceMaterial, material)
      );
    }

    this.surfaces.chunks.setShown(!material);
    this.wires.chunks.setShown(Boolean(material));
    this.currentVersion += 1;
  }

  /**
   * @param key - A texture key whose samplers now sample another texture: every batch sampling it records again.
   */
  public invalidate(key: string): void {
    for (const grouping of [this.surfaces, this.shadows, this.wires]) {
      for (const { drawing } of StaticBatches.all(grouping)) {
        drawing.forEach((batch: StaticBatch) => batch.keys.includes(key) && batch.invalidate());
      }
    }

    // A shadow changes where a caster cuts out by the texture, not where only its colour does.
    this.shadows.drawing.forEach((batch: StaticBatch, slot: number) => {
      if (batch.keys.includes(key)) {
        this.shadowChanges.touch(slot);
      }
    });
    this.currentVersion += 1;
  }

  /** Has every batch record again, for storage buffers its shaders read that were replaced by ones that grew. */
  public invalidateAll(): void {
    for (const grouping of [this.surfaces, this.shadows, this.wires]) {
      StaticBatches.all(grouping).forEach(({ drawing }) => drawing.forEach((batch: StaticBatch) => batch.invalidate()));
    }

    this.currentVersion += 1;
  }

  /**
   * @param arena - An arena whose buffers may have been replaced, which every batch over it then draws.
   */
  public refresh(arena: StaticArena): void {
    for (const grouping of [this.surfaces, this.shadows, this.wires]) {
      for (const batches of Object.values(grouping.arenas.get(arena) ?? {})) {
        batches.drawing.forEach((batch: StaticBatch) => batch.refresh());
        batches.idle.forEach((batch: StaticBatch) => batch.refresh());
      }
    }

    this.currentVersion += 1;
  }

  /**
   * @param arena - An arena going, which no batch draws any more.
   */
  public release(arena: StaticArena): void {
    for (const grouping of [this.surfaces, this.shadows, this.wires]) {
      for (const batches of Object.values(grouping.arenas.get(arena) ?? {})) {
        batches.drawing.forEach((batch: StaticBatch) => {
          grouping.chunks.detach(batch);
          batch.dispose();
        });
        batches.idle.forEach((batch: StaticBatch) => batch.dispose());
      }

      grouping.arenas.delete(arena);
    }
  }

  public dispose(): void {
    new Set([...this.surfaces.arenas.keys(), ...this.shadows.arenas.keys(), ...this.wires.arenas.keys()]).forEach(
      (arena: StaticArena) => this.release(arena)
    );

    for (const grouping of [this.surfaces, this.shadows, this.wires]) {
      grouping.chunks.clear();
      grouping.drawing.clear();
    }
  }

  /** Batches a slot's wireframe: an impostor's by its own material, every other by the one wireframe material. */
  private putWire(
    slot: number,
    arena: StaticArena,
    kind: EStaticDrawKind,
    surface: ISurfaceMaterial,
    material: Material
  ): void {
    if (surface.isImpostor) {
      StaticBatches.put(this.wires, slot, arena, kind, surface.material, surface.keys, null);
    } else {
      StaticBatches.put(this.wires, slot, arena, kind, material, [], null);
    }
  }

  private static createGrouping(
    phases: ReadonlyArray<TStaticBatchArguments>,
    scenes: ReadonlyArray<Object3D>,
    isCelled: boolean,
    isWire: boolean = false
  ): IBatchGrouping {
    return {
      arenas: new Map(),
      cells: new Map(),
      chunks: new StaticBundleChunks(scenes),
      drawing: new Map(),
      isCelled,
      isWire,
      keys: new Map(),
      phases,
    };
  }

  private static put(
    grouping: IBatchGrouping,
    slot: number,
    arena: StaticArena,
    kind: EStaticDrawKind,
    material: Material,
    keys: ReadonlyArray<string>,
    bounds: Nullable<Box3>
  ): void {
    const batches: IArenaBatches = StaticBatches.toArenaBatches(grouping, arena, kind);
    const cell: string = grouping.isCelled ? toStaticCell(bounds) : "";
    const key: string = `${material.uuid}|${cell}`;
    let batch: Maybe<StaticBatch> = batches.drawing.get(key);

    if (grouping.isCelled && bounds && cell !== STATIC_EVERYWHERE) {
      let box: Maybe<Box3> = grouping.cells.get(cell);

      if (!box) {
        box = new Box3();
        grouping.cells.set(cell, box);
      }

      box.union(bounds);
    }

    if (batch && grouping.drawing.get(slot) === batch) {
      batch.invalidate();

      return;
    }

    StaticBatches.withdraw(grouping, slot);

    if (!batch) {
      batch = batches.idle.pop() ?? new StaticBatch(arena, kind, grouping.phases, grouping.isWire);
      batch.setMaterial(material, keys);
      batches.drawing.set(key, batch);
      grouping.keys.set(batch, key);
      grouping.chunks.attach(batch, cell);
    }

    batch.add(slot);
    grouping.drawing.set(slot, batch);
  }

  private static withdraw(grouping: IBatchGrouping, slot: number): void {
    const batch: Maybe<StaticBatch> = grouping.drawing.get(slot);

    if (!batch) {
      return;
    }

    batch.remove(slot);
    grouping.drawing.delete(slot);

    if (batch.isEmpty) {
      const batches: IArenaBatches = StaticBatches.toArenaBatches(grouping, batch.arena, batch.kind);

      batches.drawing.delete(grouping.keys.get(batch) as string);
      grouping.keys.delete(batch);
      batch.setMaterial(null);
      grouping.chunks.detach(batch);
      batches.idle.push(batch);
    }
  }

  private static toArenaBatches(grouping: IBatchGrouping, arena: StaticArena, kind: EStaticDrawKind): IArenaBatches {
    let batches: Maybe<Record<EStaticDrawKind, IArenaBatches>> = grouping.arenas.get(arena);

    if (!batches) {
      batches = {
        [EStaticDrawKind.LISTED]: { drawing: new Map(), idle: [] },
        [EStaticDrawKind.SINGLE]: { drawing: new Map(), idle: [] },
      };
      grouping.arenas.set(arena, batches);
    }

    return batches[kind];
  }

  /** Every arena's batches of every kind in a grouping. */
  private static all(grouping: IBatchGrouping): Array<IArenaBatches> {
    return [...grouping.arenas.values()].flatMap((batches) => Object.values(batches));
  }
}
