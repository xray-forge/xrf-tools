import { Color, MeshStandardMaterial } from "three";

import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { applyXrayHemiShading } from "@/core/render/lib/surface/render-baked";
import { applyXrayDetailShading, IXrayDetail, IXrayDetailShading } from "@/core/render/lib/surface/render-detail";
import { applyRenderSurface, createRenderMaterial } from "@/core/render/lib/surface/render-material";
import { IRenderDetail, IRenderSurface } from "@/core/render/lib/surface/render-surface";
import { Nullable } from "@/lib/types/general";

/** Nothing a compiled level declares is metal, so the surfaces are shaded as the dielectrics xrLC assumes. */
const SURFACE_METALNESS: number = 0.0;

/**
 * Fully rough, because the deferred renderer writes one gloss for every level surface and it is `def_gloss`, two of
 * two hundred and fifty five (`shaders/r2/common.h`). A level's surfaces carry no specular worth the name, and drawing
 * them with one puts a sheen on ground and bark that the game has nowhere.
 */
const SURFACE_ROUGHNESS: number = 1.0;

/** Turns of the golden angle, which spreads consecutive shader ids rather than grouping them into near hues. */
const HUE_STEP: number = 137.508;

/** How the surfaces of a level are drawn while a toggle is on. */
export interface ILevelSurfaceOptions {
  isWireframe: boolean;
  /** Draws the surfaces with the textures the level dresses them in, or flat for comparison. */
  isTextured: boolean;
  /**
   * Whether the hemisphere occlusion xrLC baked into the level is applied, or the level is drawn under the viewer's
   * own light alone.
   */
  isLit: boolean;
  /** How much the baked hemisphere term darkens the ambient, `0` ignoring it and `1` applying it whole. */
  hemiStrength: number;
}

export const DEFAULT_LEVEL_SURFACE_OPTIONS: ILevelSurfaceOptions = {
  hemiStrength: 0.65,
  isLit: true,
  isTextured: true,
  isWireframe: false,
};

/** Everything about one drawn surface that decides how it looks. */
export interface ILevelSurface {
  /** What the level's shader table says dresses it. */
  surface: SectorSurface;
  /** What that entry's blender compiles to. */
  render: IRenderSurface;
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
 * One surface's material, with the engine shading three.js has no slot for kept beside it.
 */
export interface ILevelSurfaceMaterial {
  material: MeshStandardMaterial;
  /** Present exactly for a surface whose blender details it, since the patch is compiled in at build time. */
  detail: Nullable<IXrayDetailShading>;
}

/**
 * Builds the material one surface of a level is drawn with.
 *
 * @param surface - The surface, its blender's answer, and what its geometry carries.
 * @param textures - Where its textures come from, or null while none are loaded.
 * @param options - What the toolbar has switched on.
 * @returns A material and its detail handle, already dressed.
 */
export function createSurfaceMaterial(
  surface: ILevelSurface,
  textures: Nullable<ILevelTextureLookup>,
  options: ILevelSurfaceOptions
): ILevelSurfaceMaterial {
  const material: MeshStandardMaterial = createRenderMaterial({
    metalness: SURFACE_METALNESS,
    roughness: SURFACE_ROUGHNESS,
  });
  // Whether a surface is detailed is fixed by its blender, so the patch is installed once here rather than every time
  // the toolbar or a texture arrival re-dresses it. What changes later is the texture, which is a uniform.
  const dressed: ILevelSurfaceMaterial = {
    detail: surface.render.detail ? applyXrayDetailShading(material) : null,
    material,
  };

  // Every level surface, whether or not its table names a second texture: the patch reads the channel the engine
  // reads and does nothing at all where nothing is bound, and one patch over the whole level keeps its materials on
  // one program key.
  applyXrayHemiShading(material);

  dressSurfaceMaterial(dressed, surface, textures, options);

  return dressed;
}

/**
 * Puts a surface's own textures, its baked light, its detail and the view state onto its material.
 *
 * @param dressed - Material to dress, and the detail handle built with it.
 * @param drawn - The surface, its blender's answer, and what its geometry carries.
 * @param textures - Where its textures come from, or null while none are loaded.
 * @param options - What the toolbar has switched on.
 */
export function dressSurfaceMaterial(
  dressed: ILevelSurfaceMaterial,
  drawn: ILevelSurface,
  textures: Nullable<ILevelTextureLookup>,
  options: ILevelSurfaceOptions
): void {
  const { material } = dressed;
  const { surface } = drawn;
  const base: Nullable<ILevelTexture> =
    options.isTextured && textures && surface.textureName ? textures.get(surface.textureName) : null;
  // The row's third texture, which is the one `uber_deffer` binds as `s_hemi`. The second is R1's baked colour and
  // no deferred shader samples it; binding that one drew every lightmapped surface by an alpha that is not occlusion.
  const hemi: Nullable<ILevelTexture> = options.isLit && textures && surface.hemi ? textures.get(surface.hemi) : null;

  material.wireframe = options.isWireframe;
  material.map = base?.texture ?? null;
  // A stand-in draws at full brightness whatever the level's own light does, because a checker dimmed by a night sky
  // is just another dark surface. Lit like the rest it would say nothing; this is the one thing here that should.
  material.emissiveMap = base?.reason ? base.texture : null;
  material.emissive = new Color(material.emissiveMap ? 0xffffff : 0x000000);
  // Bound as occlusion rather than as light, which is what the deferred renderer reads out of it. See
  // `render-baked.ts` for why the file a level calls its lightmap is not one.
  material.aoMap = hemi?.texture ?? null;
  material.aoMapIntensity = options.hemiStrength;
  // Never: `v_static.color` is `(r,g,b,dir-occlusion)` and the deferred renderer reads the fourth component alone, so
  // multiplying a surface by the other three dyes it with a colour the game never shows.
  material.vertexColors = false;
  // A textured surface takes its colour from the texture, so the tint comes off or every surface is dyed.
  material.color = material.map ? new Color(0xffffff) : getShaderColor(surface.shaderId);

  dressed.detail?.setDetail(toDetailTexture(drawn, textures, options));

  applyRenderSurface(material, drawn.render);

  // One flag for every change above: `alphaTest` and whether an occlusion map is bound both change the compiled
  // program, and a material that has already drawn keeps its old one otherwise.
  material.needsUpdate = true;
}

/**
 * The detail texture to modulate with, or null wherever the modulation cannot or should not happen.
 */
function toDetailTexture(
  drawn: ILevelSurface,
  textures: Nullable<ILevelTextureLookup>,
  options: ILevelSurfaceOptions
): Nullable<IXrayDetail> {
  const detail: Nullable<IRenderDetail> = drawn.render.detail;

  if (!detail || !textures || !options.isTextured) {
    return null;
  }

  const loaded: Nullable<ILevelTexture> = textures.get(detail.reference);

  return loaded?.texture ? { scale: detail.scale, texture: loaded.texture } : null;
}
