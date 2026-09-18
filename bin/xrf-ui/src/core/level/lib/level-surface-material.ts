import { Color, MeshStandardMaterial } from "three";

import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { ILevelTexture, LevelTextureSet } from "@/core/level/lib/level-texture-set";
import { Nullable } from "@/lib/types/general";

const SURFACE_METALNESS: number = 0.0;
const SURFACE_ROUGHNESS: number = 0.9;

/** Turns of the golden angle, which spreads consecutive shader ids rather than grouping them into near hues. */
const HUE_STEP: number = 137.508;

/** Lightmaps are baked light rather than a texture, so they multiply the surface rather than replacing it. */
const LIGHTMAP_INTENSITY: number = 1.0;

/** How the surfaces of a level are drawn while a toggle is on. */
export interface ILevelSurfaceOptions {
  isWireframe: boolean;
  /** Draws the surfaces with the textures the level dresses them in, or flat for comparison. */
  isTextured: boolean;
  /** Draws each shader table entry in its own colour. */
  isSurfaceColored: boolean;
}

export const DEFAULT_LEVEL_SURFACE_OPTIONS: ILevelSurfaceOptions = {
  isSurfaceColored: false,
  isTextured: true,
  isWireframe: false,
};

/**
 * A stable colour per shader table entry, so one surface is the same colour in every sector of the level.
 *
 * @param shaderId - Entry of the level's shader table.
 * @returns A colour derived from the id rather than assigned in arrival order.
 */
export function getShaderColor(shaderId: number): Color {
  return new Color().setHSL(((shaderId * HUE_STEP) % 360) / 360, 0.45, 0.6);
}

/**
 * Builds the material one surface of a level is drawn with.
 *
 * @param surface - What the shader table says dresses it.
 * @param textures - Where its textures come from, or null while none are loaded.
 * @param options - What the toolbar has switched on.
 * @returns A material, already dressed.
 */
export function createSurfaceMaterial(
  surface: Nullable<SectorSurface>,
  textures: Nullable<LevelTextureSet>,
  options: ILevelSurfaceOptions
): MeshStandardMaterial {
  const material: MeshStandardMaterial = new MeshStandardMaterial({
    metalness: SURFACE_METALNESS,
    roughness: SURFACE_ROUGHNESS,
  });

  dressSurfaceMaterial(material, surface, textures, options);

  return material;
}

/**
 * Puts a surface's own textures and view state onto its material.
 *
 * @param material - Material to dress.
 * @param surface - What the shader table says dresses it.
 * @param textures - Where its textures come from, or null while none are loaded.
 * @param options - What the toolbar has switched on.
 */
export function dressSurfaceMaterial(
  material: MeshStandardMaterial,
  surface: Nullable<SectorSurface>,
  textures: Nullable<LevelTextureSet>,
  options: ILevelSurfaceOptions
): void {
  const base: Nullable<ILevelTexture> =
    options.isTextured && textures && surface?.textureName ? textures.get(surface.textureName) : null;
  const lightmap: Nullable<ILevelTexture> =
    options.isTextured && textures && surface?.lightmaps[0] ? textures.get(surface.lightmaps[0]) : null;

  material.wireframe = options.isWireframe;
  material.map = base?.texture ?? null;
  material.lightMap = lightmap?.texture ?? null;
  material.lightMapIntensity = LIGHTMAP_INTENSITY;
  // A textured surface takes its colour from the texture, so the tint comes off or every surface is dyed.
  material.color =
    material.map || !options.isSurfaceColored ? new Color(0xffffff) : getShaderColor(surface?.shaderId ?? 0);
  material.needsUpdate = true;
}
