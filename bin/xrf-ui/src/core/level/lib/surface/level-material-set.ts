import { MeshStandardMaterial } from "three";

import {
  createSurfaceMaterial,
  DEFAULT_LEVEL_SURFACE_OPTIONS,
  dressSurfaceMaterial,
  ILevelSurface,
  ILevelSurfaceMaterial,
  ILevelSurfaceOptions,
} from "@/core/level/lib/surface/level-surface-material";
import { ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { Maybe, Nullable } from "@/lib/types/general";

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

  private options: ILevelSurfaceOptions = DEFAULT_LEVEL_SURFACE_OPTIONS;

  /** Where a surface's textures come from, borrowed rather than owned: the loader disposes them. */
  private textures: Nullable<ILevelTextureLookup> = null;

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
  public setTextures(textures: Nullable<ILevelTextureLookup>): void {
    this.textures = textures;

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
    for (const held of this.held.values()) {
      held.dressed.material.dispose();
    }

    this.held.clear();
  }

  private dressAll(): void {
    for (const { dressed, surface } of this.held.values()) {
      dressSurfaceMaterial(dressed, surface, this.textures, this.options);
    }
  }
}
