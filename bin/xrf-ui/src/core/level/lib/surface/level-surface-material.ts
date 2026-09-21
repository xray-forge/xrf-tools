import { Color, MeshStandardMaterial, Texture } from "three";

import { SectorSurface } from "@/core/ipc/types/xrf-visual";
import { ILevelTexture, ILevelTextureLookup } from "@/core/level/lib/texture/level-texture-set";
import { applyXrayHemiShading } from "@/core/render/lib/surface/render-baked";
import { applyXrayDetailShading, IXrayDetail, IXrayDetailShading } from "@/core/render/lib/surface/render-detail";
import { applyRenderSurface, createRenderMaterial } from "@/core/render/lib/surface/render-material";
import { IRenderDetail, IRenderSurface } from "@/core/render/lib/surface/render-surface";
import { Nullable } from "@/lib/types/general";

import { ILevelSurfaceOptions } from "./level-surface-options";

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

/** Copied from rather than assigned, so dressing a textured surface allocates nothing. */
const WHITE: Color = new Color(0xffffff);
const BLACK: Color = new Color(0x000000);

/**
 * What the compiled program depends on, of everything dressing sets.
 */
const PROGRAM_MAP: number = 1;
const PROGRAM_EMISSIVE_MAP: number = 2;
const PROGRAM_AO_MAP: number = 4;
const PROGRAM_ALPHA_TEST: number = 8;

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
  /** What the material was last dressed into, of the state its program depends on. */
  program: number;
}

/**
 * What a surface's material should be wearing, decided before anything is written to it.
 */
export interface ILevelSurfaceDress {
  map: Nullable<Texture>;
  /** The base texture again wherever it is a stand-in, so a checker is not dimmed by the level's own light. */
  emissiveMap: Nullable<Texture>;
  emissive: Color;
  aoMap: Nullable<Texture>;
  aoMapIntensity: number;
  /** Under the base texture, so a textured surface is not dyed and an untextured one is told apart by hue. */
  color: Color;
  isWireframe: boolean;
  detail: Nullable<IXrayDetail>;
  /** What the compiled program depends on, against which the last dressing is compared. */
  program: number;
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
  const material: MeshStandardMaterial = createRenderMaterial(
    {
      metalness: SURFACE_METALNESS,
      roughness: SURFACE_ROUGHNESS,
    },
    surface.render
  );
  // Whether a surface is detailed is fixed by its blender, so the patch is installed once here rather than every time
  // the toolbar or a texture arrival re-dresses it. What changes later is the texture, which is a uniform.
  const dressed: ILevelSurfaceMaterial = {
    detail: surface.render.detail ? applyXrayDetailShading(material) : null,
    material,
    program: 0,
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
  applySurfaceDressing(dressed, drawn, getSurfaceDressing(drawn, textures, options));
}

/**
 * Decides what a surface's material should be wearing, without touching it.
 *
 * @param drawn - The surface, its blender's answer, and what its geometry carries.
 * @param textures - Where its textures come from, or null while none are loaded.
 * @param options - What the toolbar has switched on.
 * @returns The state to put it in, and what of that state its program depends on.
 */
export function getSurfaceDressing(
  drawn: ILevelSurface,
  textures: Nullable<ILevelTextureLookup>,
  options: ILevelSurfaceOptions
): ILevelSurfaceDress {
  const { surface } = drawn;
  const base: Nullable<ILevelTexture> =
    options.isTextured && textures && surface.textureName ? textures.get(surface.textureName) : null;
  // The row's third texture, which is the one `uber_deffer` binds as `s_hemi`. The second is R1's baked colour and
  // no deferred shader samples it; binding that one drew every lightmapped surface by an alpha that is not occlusion.
  const hemi: Nullable<ILevelTexture> = options.isLit && textures && surface.hemi ? textures.get(surface.hemi) : null;

  const map: Nullable<Texture> = base?.texture ?? null;
  // A stand-in draws at full brightness whatever the level's own light does, because a checker dimmed by a night sky
  // is just another dark surface. Lit like the rest it would say nothing; this is the one thing here that should.
  const emissiveMap: Nullable<Texture> = base?.reason ? base.texture : null;
  const aoMap: Nullable<Texture> = hemi?.texture ?? null;

  return {
    aoMap,
    aoMapIntensity: options.hemiStrength,
    // A textured surface takes its colour from the texture, so the tint comes off or every surface is dyed.
    color: map ? WHITE : getShaderColor(surface.shaderId),
    detail: toDetailTexture(drawn, textures, options),
    emissive: emissiveMap ? WHITE : BLACK,
    emissiveMap,
    isWireframe: options.isWireframe,
    map,
    program:
      (map ? PROGRAM_MAP : 0) |
      (emissiveMap ? PROGRAM_EMISSIVE_MAP : 0) |
      (aoMap ? PROGRAM_AO_MAP : 0) |
      (drawn.render.alphaTest > 0 ? PROGRAM_ALPHA_TEST : 0),
  };
}

/**
 * Writes a decided dressing onto a material, recompiling it only where its program depends on what moved.
 *
 * @param dressed - Material to dress, and the detail handle built with it.
 * @param drawn - The surface, for the state its shader compiles to.
 * @param dress - What it should be wearing.
 */
export function applySurfaceDressing(
  dressed: ILevelSurfaceMaterial,
  drawn: ILevelSurface,
  dress: ILevelSurfaceDress
): void {
  const { material } = dressed;

  material.wireframe = dress.isWireframe;
  material.map = dress.map;
  material.emissiveMap = dress.emissiveMap;
  material.emissive.copy(dress.emissive);
  // Bound as occlusion rather than as light, which is what the deferred renderer reads out of it. See
  // `render-baked.ts` for why the file a level calls its lightmap is not one.
  material.aoMap = dress.aoMap;
  material.aoMapIntensity = dress.aoMapIntensity;
  // Never: `v_static.color` is `(r,g,b,dir-occlusion)` and the deferred renderer reads the fourth component alone, so
  // multiplying a surface by the other three dyes it with a colour the game never shows.
  material.vertexColors = false;
  material.color.copy(dress.color);

  dressed.detail?.setDetail(dress.detail);

  applyRenderSurface(material, drawn.render);

  // Only where the program itself moved. Everything above is a uniform or a raster state, which three.js picks up
  // without being told; flagging them all made every sector arriving during a flight re-resolve the program of every
  // material in the level, for a program that was already the right one.
  if (dressed.program !== dress.program) {
    dressed.program = dress.program;
    material.needsUpdate = true;
  }
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
