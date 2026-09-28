import { Maybe, Nullable } from "@xrf/types";
import { Material, Texture, WebGPURenderer } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { copyTextures } from "#/internals/texture-copies";
import { TSurfaceArrayTargets } from "#/material/surface-array-targets";
import { ISurfaceBatchMaterial } from "#/material/surface-batch-material";
import { toSurfaceCompositing } from "#/material/surface-compositing";
import { createSurfaceBatchMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ESurfaceSlot, SURFACE_SLOTS } from "#/material/surface-slot";
import { ISurfaceValues, toSurfaceValues } from "#/material/surface-values";
import { ISurfaceVariant, toSampledSlots, toSurfaceVariant, toSurfaceVariantKey } from "#/material/surface-variant";
import { RendererTextures } from "#/texture/renderer-textures";
import { ITextureArrayFlush } from "#/texture/texture-array-flush";
import { TextureArrays } from "#/texture/texture-arrays";
import { ITextureLayer } from "#/texture/texture-layer";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { SurfaceTable } from "#/uniforms/surface-table";

/** The slots a static batch samples from arrays, where their textures are of a class one holds: every static one. */
export const SURFACE_ARRAY_SLOTS: ReadonlyArray<ESurfaceSlot> = [
  ESurfaceSlot.BASE,
  ESurfaceSlot.DETAIL,
  ESurfaceSlot.BUMP,
  ESurfaceSlot.BUMP_COMPANION,
  ESurfaceSlot.HEMI,
];

/** A shared material, and how many surfaces draw by it. */
interface ISharedMaterial {
  key: string;
  batch: ISurfaceBatchMaterial;
  /** The keys of the arrays it samples, which bundles binding it record again for when one is replaced. */
  arrayKeys: ReadonlyArray<string>;
  users: number;
}

/** What one batched view of a surface holds, all of it kept until nothing draws the view. */
interface ISurfaceView {
  /** Each array slot it samples, with its key and where that is held. */
  layers: ReadonlyArray<readonly [ESurfaceSlot, string, ITextureLayer]>;
  shared: ISharedMaterial;
  row: number;
}

/** What batching one surface takes and holds. */
interface ISurfaceTracking {
  variant: ISurfaceVariant;
  values: ISurfaceValues;
  textures: IRendererSurface["textures"];
  /** Its array slots, each with its texture's key. */
  arrayed: ReadonlyArray<readonly [ESurfaceSlot, string]>;
  /** The view it is batched by now, or null while it draws its static batches by its own material. */
  view: Nullable<ISurfaceView>;
}

/**
 * Which surfaces a static batch draws by a material they share: one variant and every texture but their array slots',
 * those held in arrays of their class, each surface's numbers and layers a row of the surface table. A view left keeps
 * its row, layers and shared material until the release handed back for it runs, once nothing draws it.
 */
export class SurfaceBatching {
  private readonly textures: RendererTextures;
  private readonly uniforms: RendererUniforms;
  private readonly programs: SurfacePrograms;
  private readonly table: SurfaceTable;
  private readonly arrays: TextureArrays;
  private readonly onInvalidated: (key: string) => void;

  private readonly tracked: Map<ISurfaceMaterial, ISurfaceTracking> = new Map();
  /** The surfaces sampling each key from an array slot. */
  private readonly users: Map<string, Set<ISurfaceMaterial>> = new Map();
  private readonly shared: Map<string, ISharedMaterial> = new Map();
  /** Shared materials no surface draws by any more, disposed once no batch draws them. */
  private readonly idle: Set<ISharedMaterial> = new Set();
  /** Keys copied into their layers whose own textures something still draws, let go of once nothing does. */
  private readonly unevicted: Set<string> = new Set();
  private tableVersion: number = 0;

  /**
   * @param textures - Where the textures are bound from.
   * @param uniforms - What the frame's shaders read, the surface table among them.
   * @param programs - The shaders the surfaces share.
   * @param onInvalidated - Told a key as a texture's where what bundles bind under it was replaced: an array, or the
   *   table.
   */
  public constructor(
    textures: RendererTextures,
    uniforms: RendererUniforms,
    programs: SurfacePrograms,
    onInvalidated: (key: string) => void
  ) {
    this.textures = textures;
    this.uniforms = uniforms;
    this.programs = programs;
    this.table = uniforms.surfaceTable;
    this.onInvalidated = onInvalidated;
    this.arrays = new TextureArrays(onInvalidated);
  }

  /** Layers the device allows an array, which the device says once it is open. */
  public set layerLimit(limit: number) {
    this.arrays.layerLimit = limit;
  }

  /**
   * @param material - A surface's material, drawing from now on.
   * @param surface - What it was built from.
   * @returns Whether it draws its static batches by a shared material from now on.
   */
  public track(material: ISurfaceMaterial, surface: IRendererSurface): boolean {
    if (this.tracked.has(material)) {
      return false;
    }

    const variant: ISurfaceVariant = toSurfaceVariant(surface);

    if (variant.pass !== ERendererPass.DEFERRED || variant.isImpostor || toSurfaceCompositing(surface)) {
      return false;
    }

    const sampled: ReadonlyArray<ESurfaceSlot> = toSampledSlots(variant);
    const arrayed: Array<readonly [ESurfaceSlot, string]> = SURFACE_ARRAY_SLOTS.flatMap((slot: ESurfaceSlot) => {
      const key: Maybe<string> = surface.textures[slot];

      return sampled.includes(slot) && key ? [[slot, key] as const] : [];
    });

    if (!arrayed.length) {
      return false;
    }

    const tracking: ISurfaceTracking = {
      arrayed,
      textures: surface.textures,
      values: toSurfaceValues(surface),
      variant,
      view: null,
    };

    this.tracked.set(material, tracking);

    for (const [, key] of arrayed) {
      let users: Maybe<Set<ISurfaceMaterial>> = this.users.get(key);

      if (!users) {
        users = new Set();
        this.users.set(key, users);
      }

      users.add(material);
    }

    return this.evaluate(material, tracking);
  }

  /**
   * @param material - A surface's material, which nothing draws from the change now running on.
   * @returns What lets go of the view it was batched by, once that change applies; null for none.
   */
  public untrack(material: ISurfaceMaterial): Nullable<() => void> {
    const tracking: Maybe<ISurfaceTracking> = this.tracked.get(material);

    if (!tracking) {
      return null;
    }

    this.tracked.delete(material);

    for (const [, key] of tracking.arrayed) {
      const users: Maybe<Set<ISurfaceMaterial>> = this.users.get(key);

      users?.delete(material);

      if (users && !users.size) {
        this.users.delete(key);
      }
    }

    return this.detach(material, tracking);
  }

  /**
   * @param key - A texture's key whose samplers were pointed at another texture.
   * @returns The surfaces whose batched view changed with it, which whatever draws them has to be built again for,
   *   each with what lets go of the view it was batched by, once that is done.
   */
  public rebind(key: string): Array<readonly [ISurfaceMaterial, Nullable<() => void>]> {
    const users: Maybe<Set<ISurfaceMaterial>> = this.users.get(key);

    if (!users?.size) {
      return [];
    }

    // Of the class it was held as: its layer is copied again, and nothing else changes.
    if (this.arrays.refresh(key, this.textures.getUploaded(key))) {
      return [];
    }

    // Of another class, or nothing: every view made from now on holds it as it is now, while the views drawn keep its
    // layer as it was, and every other key they hold, until they are let go.
    this.arrays.detach(key);

    const changed: Array<readonly [ISurfaceMaterial, Nullable<() => void>]> = [];

    for (const material of users) {
      const tracking: ISurfaceTracking = this.tracked.get(material) as ISurfaceTracking;
      const release: Nullable<() => void> = this.detach(material, tracking);

      // Built again wherever it was or is batched: its row is another now, and its draws name their row.
      if (this.evaluate(material, tracking) || release) {
        changed.push([material, release]);
      }
    }

    return changed;
  }

  /** Whether any shared material waits for no batch to draw it, to be disposed. */
  public get hasIdle(): boolean {
    return this.idle.size > 0;
  }

  /**
   * Disposes every shared material no surface draws by and no batch draws.
   *
   * @param drawn - Every material something draws.
   */
  public retire(drawn: ReadonlySet<Material>): void {
    for (const shared of this.idle) {
      if (!drawn.has(shared.batch.material)) {
        this.idle.delete(shared);
        this.shared.delete(shared.key);
        shared.batch.dispose();
      }
    }
  }

  /**
   * Copies the layers waiting into their arrays and the rows written to the GPU, before the frame draws.
   *
   * @param renderer - The renderer drawing.
   */
  public flush(renderer: WebGPURenderer): void {
    // Fitted once a level has stopped streaming layers in, which it does a layer at a time.
    this.arrays.compact(performance.now());

    const { copies, disposals, evicted }: ITextureArrayFlush = this.arrays.flush();

    // A key whose copy could not be made keeps its own texture, to be copied from again next flush.
    const retried: Set<string> = this.arrays.retry(copyTextures(renderer, copies));
    const evictions: Array<readonly [string, Texture]> = [];

    evicted.forEach((key: string) => this.unevicted.add(key));

    // Sent, so the GPU finishes reading them first. A key copied lets its own texture go, its layer drawing it, once
    // nothing draws the texture itself; a bundle still sampling that records again.
    for (const key of this.unevicted) {
      if (retried.has(key)) {
        continue;
      }

      const texture: Nullable<Texture> = this.arrays.holds(key) ? this.textures.evict(key) : null;

      if (texture) {
        evictions.push([key, texture]);
      }

      // Kept up by what draws it, or its latest put not up yet: tried again with the next flush. One let go already
      // stays up once something asks for it again.
      if (texture || !this.arrays.holds(key) || this.textures.isEvicted(key)) {
        this.unevicted.delete(key);
      }
    }

    this.programs.nodes.forget(
      new Set([...disposals, ...evictions.map(([, texture]: readonly [string, Texture]) => texture)])
    );
    disposals.forEach((texture: Texture) => texture.dispose());
    evictions.forEach(([key]: readonly [string, Texture]) => this.onInvalidated(key));
    this.table.flush();

    if (this.table.version !== this.tableVersion) {
      this.tableVersion = this.table.version;
      this.onInvalidated(this.table.key);
    }
  }

  public dispose(): void {
    [...this.tracked.keys()].forEach((material: ISurfaceMaterial) => this.untrack(material)?.());
    this.shared.forEach((shared: ISharedMaterial) => shared.batch.dispose());
    this.shared.clear();
    this.idle.clear();
    this.unevicted.clear();
    this.arrays.dispose();
  }

  /**
   * Batches a surface with no view once its array slots' textures are up, each held in an array of its class or
   * sampled as its own where none holds it.
   *
   * @returns Whether it is batched now.
   */
  private evaluate(material: ISurfaceMaterial, tracking: ISurfaceTracking): boolean {
    const layers: Array<readonly [ESurfaceSlot, string, ITextureLayer]> = [];
    // A key held already draws from its layer, its own texture on the GPU or not. Every other is asked for, not only
    // the first missing, so every one evicted goes up again in the same uploads.
    const missing: ReadonlyArray<readonly [ESurfaceSlot, string]> = tracking.arrayed.filter(
      ([, key]: readonly [ESurfaceSlot, string]) => !this.arrays.holds(key) && !this.textures.getUploaded(key)
    );

    if (missing.length) {
      return false;
    }

    for (const [slot, key] of tracking.arrayed) {
      const held: Nullable<ITextureLayer> = this.arrays.retain(key) ?? this.claim(key);

      if (held) {
        layers.push([slot, key, held]);
      }
    }

    const shared: ISharedMaterial = this.getShared(tracking, layers);
    const row: number = this.table.allocate();

    this.writeRow(row, tracking.values, layers);
    this.idle.delete(shared);
    shared.users += 1;
    tracking.view = { layers, row, shared };
    material.batched = this.createView(material, shared, row);

    return true;
  }

  /** A key's layer claimed from its uploaded texture, or null for a texture of no class an array holds. */
  private claim(key: string): Nullable<ITextureLayer> {
    const texture: Nullable<Texture> = this.textures.getUploaded(key);

    return texture ? this.arrays.claim(key, texture) : null;
  }

  /**
   * Takes a surface off the view it is batched by: from now on it draws its static batches by its own material.
   *
   * @returns What lets go of the view, its layers, its row and its share of the shared material, once nothing draws
   *   it; null for a surface batched by none.
   */
  private detach(material: ISurfaceMaterial, tracking: ISurfaceTracking): Nullable<() => void> {
    const { view } = tracking;

    tracking.view = null;
    material.batched = null;

    if (!view) {
      return null;
    }

    let isReleased: boolean = false;

    return () => {
      if (!isReleased) {
        isReleased = true;
        this.release(view);
      }
    };
  }

  /** Lets go of a view nothing draws: the shared material it drew by goes idle once no view does. */
  private release({ layers, row, shared }: ISurfaceView): void {
    layers.forEach(([, key, held]: readonly [ESurfaceSlot, string, ITextureLayer]) => this.arrays.release(key, held));
    this.table.release(row);

    if (--shared.users === 0) {
      this.idle.add(shared);
    }
  }

  /** The shared material of a surface's variant, own textures and arrays, made the first time any surface asks. */
  private getShared(
    tracking: ISurfaceTracking,
    layers: ReadonlyArray<readonly [ESurfaceSlot, string, ITextureLayer]>
  ): ISharedMaterial {
    const arrayed: ReadonlyArray<ESurfaceSlot> = layers.map(
      ([slot]: readonly [ESurfaceSlot, string, ITextureLayer]) => slot
    );
    const own: string = toSampledSlots(tracking.variant)
      .filter((slot: ESurfaceSlot) => !arrayed.includes(slot))
      .map((slot: ESurfaceSlot) => `${slot}=${tracking.textures[slot] ?? ""}`)
      .join(",");
    const key: string = [
      toSurfaceVariantKey(tracking.variant),
      own,
      layers
        .map(([slot, , held]: readonly [ESurfaceSlot, string, ITextureLayer]) => `${slot}@${held.array.key}`)
        .join(","),
    ].join("|");
    let shared: Maybe<ISharedMaterial> = this.shared.get(key);

    if (!shared) {
      const arrays: TSurfaceArrayTargets = Object.fromEntries(
        layers.map(([slot, , held]: readonly [ESurfaceSlot, string, ITextureLayer]) => [slot, held.array.target])
      ) as TSurfaceArrayTargets;

      shared = {
        arrayKeys: layers.map(([, , held]: readonly [ESurfaceSlot, string, ITextureLayer]) => held.array.key),
        batch: createSurfaceBatchMaterial(
          tracking.variant,
          tracking.textures,
          arrays,
          this.textures,
          this.uniforms,
          this.programs
        ),
        key,
        users: 0,
      };
      this.shared.set(key, shared);
    }

    return shared;
  }

  private writeRow(
    row: number,
    values: ISurfaceValues,
    layers: ReadonlyArray<readonly [ESurfaceSlot, string, ITextureLayer]>
  ): void {
    const words: Array<number> = SURFACE_SLOTS.map(() => 0);

    layers.forEach(
      ([slot, , held]: readonly [ESurfaceSlot, string, ITextureLayer]) =>
        (words[SURFACE_SLOTS.indexOf(slot)] = held.layer)
    );
    this.table.write(row, {
      alphaReference: values.alphaReference,
      color: [values.color.x, values.color.y, values.color.z],
      detailScale: values.detailScale,
      layers: words,
      slice: values.slice,
      tiling: values.tiling,
    });
  }

  /**
   * A surface as a static batch draws it by a shared material: its row, the shared material, and its own for its plain
   * parts.
   */
  private createView(material: ISurfaceMaterial, shared: ISharedMaterial, row: number): ISurfaceMaterial {
    return {
      ...material,
      batched: null,
      dispose: () => {},
      // Its array slots' keys are the arrays': its layers draw them, their own textures on the GPU or not.
      keys: [...shared.batch.keys, ...shared.arrayKeys, this.table.key],
      material: shared.batch.material,
      row,
      shadow: shared.batch.shadow ?? material.shadow,
      shadowKeys: shared.batch.shadow
        ? [...shared.batch.shadowKeys, ...shared.arrayKeys, this.table.key]
        : material.shadowKeys,
    };
  }
}
