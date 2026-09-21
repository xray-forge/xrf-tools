import { MeshStandardMaterial } from "three";

import { listSurfaceTextures } from "@/core/level/lib/sector/level-sector-textures";
import { ILevelTextureSource, TLevelTextureChange } from "@/core/level/lib/texture/level-texture-set";
import { Maybe, Nullable } from "@/lib/types/general";

import {
  applySurfaceDressing,
  createSurfaceMaterial,
  getSurfaceDressing,
  ILevelSurface,
  ILevelSurfaceMaterial,
} from "./level-surface-material";
import { DEFAULT_LEVEL_SURFACE_OPTIONS, ILevelSurfaceOptions } from "./level-surface-options";

/**
 * What makes two surfaces the same material, which is the shader table row they were dressed from.
 */
function getSurfaceKey(surface: ILevelSurface): string {
  return String(surface.surface.shaderId);
}

/** One material, and the surface it was dressed from, so a view toggle can dress it again. */
interface IHeldMaterial {
  dressed: ILevelSurfaceMaterial;
  surface: ILevelSurface;
}

/**
 * Owns the materials a level's surfaces are drawn with, shared by every sector that names one.
 */
export class LevelMaterialSet {
  private readonly held: Map<string, IHeldMaterial> = new Map();

  /**
   * Which materials each texture reference dresses.
   */
  private readonly byReference: Map<string, Set<string>> = new Map();

  private options: ILevelSurfaceOptions = DEFAULT_LEVEL_SURFACE_OPTIONS;

  /** Where a surface's textures come from, borrowed rather than owned: the loader disposes them. */
  private textures: Nullable<ILevelTextureSource> = null;

  /** Stops this set hearing about the textures it last took, for when it takes another level's. */
  private unsubscribe: Nullable<() => void> = null;

  /**
   * @returns How many distinct materials are held, which is what a sector's draws really cost.
   */
  public get size(): number {
    return this.held.size;
  }

  /**
   * Takes the set a later material is dressed from, and re-dresses what is already held.
   *
   * @param textures - The open level's textures, owned by the loader.
   */
  public setTextures(textures: Nullable<ILevelTextureSource>): void {
    this.unsubscribe?.();

    this.textures = textures;
    this.unsubscribe = textures?.subscribe((changed: TLevelTextureChange) => this.redress(changed)) ?? null;

    this.dressAll();
  }

  /**
   * The material one surface is drawn with, made once and shared from then on.
   *
   * @param surface - The surface, its blender's answer, and what its geometry carries.
   * @returns The material, already dressed in the current view state.
   */
  public claim(surface: ILevelSurface): MeshStandardMaterial {
    const key: string = getSurfaceKey(surface);
    const held: Maybe<IHeldMaterial> = this.held.get(key);

    if (held) {
      return held.dressed.material;
    }

    const dressed: ILevelSurfaceMaterial = createSurfaceMaterial(surface, this.textures, this.options);

    this.held.set(key, { dressed, surface });

    for (const texture of listSurfaceTextures(surface)) {
      let keys: Maybe<Set<string>> = this.byReference.get(texture.reference);

      if (!keys) {
        keys = new Set();
        this.byReference.set(texture.reference, keys);
      }

      keys.add(key);
    }

    return dressed.material;
  }

  /**
   * Disposes every material outside the given set.
   *
   * @param surfaces - Everything the drawn sectors still name.
   */
  public retain(surfaces: Iterable<ILevelSurface>): void {
    const wanted: Set<string> = new Set(Array.from(surfaces, getSurfaceKey));

    for (const [key, held] of Array.from(this.held)) {
      if (!wanted.has(key)) {
        held.dressed.material.dispose();
        this.held.delete(key);
        this.forget(key, held.surface);
      }
    }
  }

  /**
   * Takes how much of the baked hemisphere term to apply, which the lighting owns rather than the toolbar.
   *
   * @param strength - Zero ignoring the term, one applying it whole.
   */
  public setHemiStrength(strength: number): void {
    this.options = { ...this.options, hemiStrength: strength };

    this.dressAll();
  }

  /**
   * Applies the view toggles to every material held.
   *
   * @param options - What the toolbar has switched on.
   */
  public applyViewOptions(options: ILevelSurfaceOptions): void {
    // The hemisphere strength is the lighting's and reaches here through `setHemiStrength`; a toolbar toggle carries
    // whatever the defaults had in it, and taking that would put the slider back every time a checkbox moved.
    this.options = { ...options, hemiStrength: this.options.hemiStrength };

    this.dressAll();
  }

  /** Releases every material, for teardown and for swapping levels. */
  public dispose(): void {
    this.unsubscribe?.();
    this.unsubscribe = null;
    this.textures = null;

    for (const held of this.held.values()) {
      held.dressed.material.dispose();
    }

    this.held.clear();
    this.byReference.clear();
  }

  /**
   * Re-dresses the materials a texture change reaches.
   *
   * @param changed - References whose upload moved, or null where the whole set went.
   */
  private redress(changed: TLevelTextureChange): void {
    // The whole set went, which is a level opening or closing: there is no reference to look anything up by.
    if (!changed) {
      this.dressAll();

      return;
    }

    const keys: Set<string> = new Set();

    for (const reference of changed) {
      for (const key of this.byReference.get(reference) ?? []) {
        keys.add(key);
      }
    }

    for (const key of keys) {
      this.dress(key);
    }
  }

  private dressAll(): void {
    for (const key of this.held.keys()) {
      this.dress(key);
    }
  }

  private dress(key: string): void {
    const held: Maybe<IHeldMaterial> = this.held.get(key);

    if (held) {
      applySurfaceDressing(held.dressed, held.surface, getSurfaceDressing(held.surface, this.textures, this.options));
    }
  }

  /** Takes a disposed material out of the index, dropping a reference nothing is left dressing. */
  private forget(key: string, surface: ILevelSurface): void {
    for (const texture of listSurfaceTextures(surface)) {
      const keys: Maybe<Set<string>> = this.byReference.get(texture.reference);

      if (keys?.delete(key) && !keys.size) {
        this.byReference.delete(texture.reference);
      }
    }
  }
}
