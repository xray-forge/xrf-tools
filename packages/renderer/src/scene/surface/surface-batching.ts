import { Maybe, Nullable } from "@xrf/types";
import { Material, Texture, WebGPURenderer } from "three/webgpu";

import { ERendererPass, IRendererSurface } from "#/contract/scene/renderer-surface";
import { toSurfaceCompositing } from "#/material/surface-compositing";
import { createSurfaceBatchMaterial, ISurfaceBatchMaterial, ISurfaceMaterial } from "#/material/surface-material";
import { SurfacePrograms } from "#/material/surface-programs";
import { ESurfaceSlot, SURFACE_SLOTS, TSurfaceArrayTargets } from "#/material/surface-slot";
import { ISurfaceValues, toSurfaceValues } from "#/material/surface-values";
import { ISurfaceVariant, toSampledSlots, toSurfaceVariant, toSurfaceVariantKey } from "#/material/surface-variant";
import { RendererTextures } from "#/texture/renderer-textures";
import { ITextureLayer, TextureArrays } from "#/texture/texture-arrays";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";
import { SURFACE_NO_ROW, SurfaceTable } from "#/uniforms/surface-table";

/** The slots a static batch samples from arrays rather than from each surface's own texture: the lightmaps. */
export const SURFACE_ARRAY_SLOTS: ReadonlyArray<ESurfaceSlot> = [ESurfaceSlot.HEMI];

/** A shared material, and how many surfaces draw by it. */
interface ISharedMaterial {
  key: string;
  batch: ISurfaceBatchMaterial;
  /** The keys of the arrays it samples, which bundles binding it record again for when one is replaced. */
  arrayKeys: ReadonlyArray<string>;
  users: number;
}

/** What batching one surface takes and holds. */
interface ISurfaceTracking {
  variant: ISurfaceVariant;
  values: ISurfaceValues;
  textures: IRendererSurface["textures"];
  /** Its array slots, each with its texture's key. */
  arrayed: ReadonlyArray<readonly [ESurfaceSlot, string]>;
  /** The keys it holds a layer of. */
  claims: Array<string>;
  shared: Nullable<ISharedMaterial>;
  row: number;
}

/**
 * Which surfaces a static batch draws by a material they share: one variant and every texture but their array slots',
 * those held in arrays of their class, each surface's numbers and layers a row of the surface table.
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

    this.tracked.set(material, {
      arrayed,
      claims: [],
      row: SURFACE_NO_ROW,
      shared: null,
      textures: surface.textures,
      values: toSurfaceValues(surface),
      variant,
    });

    for (const [, key] of arrayed) {
      let users: Maybe<Set<ISurfaceMaterial>> = this.users.get(key);

      if (!users) {
        users = new Set();
        this.users.set(key, users);
      }

      users.add(material);
    }

    return this.evaluate(material);
  }

  /**
   * @param material - A surface's material, drawn no more.
   */
  public untrack(material: ISurfaceMaterial): void {
    const tracking: Maybe<ISurfaceTracking> = this.tracked.get(material);

    if (!tracking) {
      return;
    }

    this.unbatch(material, tracking);
    this.tracked.delete(material);

    for (const [, key] of tracking.arrayed) {
      const users: Maybe<Set<ISurfaceMaterial>> = this.users.get(key);

      users?.delete(material);

      if (users && !users.size) {
        this.users.delete(key);
      }
    }
  }

  /**
   * @param key - A texture's key whose samplers were pointed at another texture.
   * @returns The surfaces whose batched view changed with it, which whatever draws them has to be built again for.
   */
  public rebind(key: string): Array<ISurfaceMaterial> {
    const users: Maybe<Set<ISurfaceMaterial>> = this.users.get(key);

    if (!users?.size) {
      return [];
    }

    // Of the class it was held as: its layer is copied again, and nothing else changes.
    if (this.arrays.refresh(key, this.textures.getUploaded(key))) {
      return [];
    }

    const materials: Array<ISurfaceMaterial> = [...users];
    const before: Array<Nullable<ISharedMaterial>> = materials.map((it) => this.tracked.get(it)?.shared ?? null);

    // Every user lets it go before any claims it again, so a texture of another class is held as that one.
    materials.forEach((material: ISurfaceMaterial) =>
      this.unbatch(material, this.tracked.get(material) as ISurfaceTracking)
    );
    materials.forEach((material: ISurfaceMaterial) => this.evaluate(material));

    // Built again wherever it was or is batched: its row may be another now, and its draws name their row.
    return materials.filter(
      (material: ISurfaceMaterial, index: number) => before[index] !== null || material.batched !== null
    );
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
    this.arrays.flush(renderer);
    this.table.flush();

    if (this.table.version !== this.tableVersion) {
      this.tableVersion = this.table.version;
      this.onInvalidated(this.table.key);
    }
  }

  public dispose(): void {
    [...this.tracked.keys()].forEach((material: ISurfaceMaterial) => this.untrack(material));
    this.shared.forEach((shared: ISharedMaterial) => shared.batch.dispose());
    this.shared.clear();
    this.idle.clear();
    this.arrays.dispose();
  }

  /** Batches a surface where its array slots' textures are up and held, and unbatches it where they are not. */
  private evaluate(material: ISurfaceMaterial): boolean {
    const tracking: ISurfaceTracking = this.tracked.get(material) as ISurfaceTracking;
    const layers: Array<readonly [ESurfaceSlot, string, ITextureLayer]> = [];

    for (const [slot, key] of tracking.arrayed) {
      const texture: Nullable<Texture> = this.textures.getUploaded(key);
      const held: Nullable<ITextureLayer> = texture ? this.arrays.claim(key, texture) : null;

      if (!held) {
        layers.forEach(([, claimed]) => this.arrays.release(claimed));

        return this.unbatch(material, tracking);
      }

      layers.push([slot, key, held]);
    }

    // Claimed again before the claims before are let go, so a layer both hold stays where it is.
    tracking.claims.forEach((key: string) => this.arrays.release(key));
    tracking.claims = layers.map(([, key]) => key);

    const shared: ISharedMaterial = this.getShared(tracking, layers);

    if (tracking.row === SURFACE_NO_ROW) {
      tracking.row = this.table.allocate();
    }

    this.writeRow(tracking, layers);

    if (tracking.shared === shared) {
      return false;
    }

    this.leave(tracking);
    this.idle.delete(shared);
    shared.users += 1;
    tracking.shared = shared;
    material.batched = this.createView(material, shared, tracking.row);

    return true;
  }

  /** Lets go of what a surface holds for its batching; it draws its static batches by its own material again. */
  private unbatch(material: ISurfaceMaterial, tracking: ISurfaceTracking): boolean {
    const wasBatched: boolean = tracking.shared !== null;

    tracking.claims.forEach((key: string) => this.arrays.release(key));
    tracking.claims = [];
    this.leave(tracking);

    if (tracking.row !== SURFACE_NO_ROW) {
      this.table.release(tracking.row);
      tracking.row = SURFACE_NO_ROW;
    }

    material.batched = null;

    return wasBatched;
  }

  /** Takes a surface out of the shared material it drew by, which goes idle once no surface does. */
  private leave(tracking: ISurfaceTracking): void {
    const previous: Nullable<ISharedMaterial> = tracking.shared;

    tracking.shared = null;

    if (previous && --previous.users === 0) {
      this.idle.add(previous);
    }
  }

  /** The shared material of a surface's variant, own textures and arrays, made the first time any surface asks. */
  private getShared(
    tracking: ISurfaceTracking,
    layers: ReadonlyArray<readonly [ESurfaceSlot, string, ITextureLayer]>
  ): ISharedMaterial {
    const arrayed: ReadonlyArray<ESurfaceSlot> = layers.map(([slot]) => slot);
    const own: string = toSampledSlots(tracking.variant)
      .filter((slot: ESurfaceSlot) => !arrayed.includes(slot))
      .map((slot: ESurfaceSlot) => `${slot}=${tracking.textures[slot] ?? ""}`)
      .join(",");
    const key: string = [
      toSurfaceVariantKey(tracking.variant),
      own,
      layers.map(([slot, , held]) => `${slot}@${held.array.key}`).join(","),
    ].join("|");
    let shared: Maybe<ISharedMaterial> = this.shared.get(key);

    if (!shared) {
      const arrays: TSurfaceArrayTargets = Object.fromEntries(
        layers.map(([slot, , held]) => [slot, held.array.target])
      ) as TSurfaceArrayTargets;

      shared = {
        arrayKeys: layers.map(([, , held]) => held.array.key),
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
    tracking: ISurfaceTracking,
    layers: ReadonlyArray<readonly [ESurfaceSlot, string, ITextureLayer]>
  ): void {
    const { values } = tracking;
    const words: Array<number> = SURFACE_SLOTS.map(() => 0);

    layers.forEach(([slot, , held]) => (words[SURFACE_SLOTS.indexOf(slot)] = held.layer));
    this.table.write(tracking.row, {
      alphaReference: values.alphaReference,
      color: [values.color.x, values.color.y, values.color.z],
      detailScale: values.detailScale,
      layers: words,
      slice: values.slice,
      tiling: values.tiling,
    });
  }

  /** A surface as a static batch draws it by a shared material: its row, the shared material, and its own otherwise. */
  private createView(material: ISurfaceMaterial, shared: ISharedMaterial, row: number): ISurfaceMaterial {
    return {
      ...material,
      batched: null,
      dispose: () => {},
      // Its own keys too: its array slots' textures have to be up before it draws, and their swaps record it again.
      keys: [...material.keys, ...shared.batch.keys, ...shared.arrayKeys, this.table.key],
      material: shared.batch.material,
      plainMaterial: material.material,
      row,
    };
  }
}
