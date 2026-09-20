import { Color, MeshStandardMaterial } from "three";

import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/level-texture-set";
import { applyRenderSurface, createRenderMaterial } from "@/core/render/lib/render-material";
import { IRenderSurface, OPAQUE_RENDER_SURFACE } from "@/core/render/lib/render-surface";
import { Nullable } from "@/lib/types/general";

/** Nothing a compiled level declares is metal, so the surfaces are shaded as the dielectrics xrLC assumes. */
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
  /** Whether a surface whose shader reads alpha is cut out and blended as the engine does, or drawn solid. */
  isAlphaVisible: boolean;
  /** Whether the light xrLC baked into the level is applied, or the level is drawn under the viewer's own light alone. */
  isLit: boolean;
}

export const DEFAULT_LEVEL_SURFACE_OPTIONS: ILevelSurfaceOptions = {
  isAlphaVisible: true,
  isLit: true,
  isSurfaceColored: false,
  isTextured: true,
  isWireframe: false,
};

/** Everything about one drawn surface that decides how it looks. */
export interface ILevelSurface {
  /** What the level's shader table says dresses it. */
  surface: SectorSurface;
  /** What that entry's blender compiles to. */
  render: IRenderSurface;
  /** Whether the geometry drawn with it carries the vertex colour xrLC baked, which not every declaration does. */
  hasVertexColors: boolean;
}

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
 * @param surface - The surface, its blender's answer, and what its geometry carries.
 * @param textures - Where its textures come from, or null while none are loaded.
 * @param options - What the toolbar has switched on.
 * @returns A material, already dressed.
 */
export function createSurfaceMaterial(
  surface: ILevelSurface,
  textures: Nullable<ILevelTextureLookup>,
  options: ILevelSurfaceOptions
): MeshStandardMaterial {
  const material: MeshStandardMaterial = createRenderMaterial({
    metalness: SURFACE_METALNESS,
    roughness: SURFACE_ROUGHNESS,
  });

  dressSurfaceMaterial(material, surface, textures, options);

  return material;
}

/**
 * Puts a surface's own textures, its baked light and the view state onto its material.
 *
 * @param material - Material to dress.
 * @param drawn - The surface, its blender's answer, and what its geometry carries.
 * @param textures - Where its textures come from, or null while none are loaded.
 * @param options - What the toolbar has switched on.
 */
export function dressSurfaceMaterial(
  material: MeshStandardMaterial,
  drawn: ILevelSurface,
  textures: Nullable<ILevelTextureLookup>,
  options: ILevelSurfaceOptions
): void {
  const { surface } = drawn;
  const base: Nullable<ILevelTexture> =
    options.isTextured && textures && surface.textureName ? textures.get(surface.textureName) : null;
  const lightmap: Nullable<ILevelTexture> =
    options.isLit && textures && surface.lightmaps[0] ? textures.get(surface.lightmaps[0]) : null;

  material.wireframe = options.isWireframe;
  material.map = base?.texture ?? null;
  material.lightMap = lightmap?.texture ?? null;
  material.lightMapIntensity = LIGHTMAP_INTENSITY;
  // Only where the geometry carries the attribute: switching it on without one leaves the shader reading a buffer
  // that is not bound, which draws nothing at all rather than drawing it unlit.
  material.vertexColors = options.isLit && drawn.hasVertexColors;
  // A textured surface takes its colour from the texture, so the tint comes off or every surface is dyed.
  material.color = material.map || !options.isSurfaceColored ? new Color(0xffffff) : getShaderColor(surface.shaderId);

  applyRenderSurface(material, options.isAlphaVisible ? drawn.render : OPAQUE_RENDER_SURFACE);

  // One flag for every change above: `alphaTest` and `vertexColors` both change the compiled program, and a material
  // that has already drawn keeps its old one otherwise.
  material.needsUpdate = true;
}
