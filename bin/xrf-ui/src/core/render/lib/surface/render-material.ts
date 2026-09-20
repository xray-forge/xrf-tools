import { MeshStandardMaterial, OneMinusSrcAlphaFactor, SrcAlphaFactor } from "three";

import { applyXrayGlossShading, XRAY_DEFAULT_GLOSS } from "@/core/render/lib/surface/render-gloss";
import { IRenderSurface, OPAQUE_RENDER_SURFACE } from "@/core/render/lib/surface/render-surface";

/**
 * How far towards the viewer a composited surface is pulled, in the depth buffer's own slope-scaled units.
 */
const DECAL_POLYGON_OFFSET: number = -1;

/** What a surface is made of, beyond what its shader says. X-Ray authors no metalness, so nothing here is guessed. */
export interface IRenderMaterialOptions {
  metalness: number;
  roughness: number;
  /** Tint under the base texture, white wherever one is bound or the surface would be dyed by it. */
  color?: number;
  /** What the surface's own shader writes into the gloss channel, `def_gloss` for everything that writes no other. */
  gloss?: number;
}

/**
 * Builds the material an X-Ray surface is drawn with.
 *
 * @param options - How the surface is shaded beyond its shader's own answer.
 * @param surface - Material state its shader compiles to, applied before the material is ever drawn.
 * @returns A material, already in the state its shader asks for.
 */
export function createRenderMaterial(
  options: IRenderMaterialOptions,
  surface: IRenderSurface = OPAQUE_RENDER_SURFACE
): MeshStandardMaterial {
  const material: MeshStandardMaterial = new MeshStandardMaterial({
    color: options.color ?? 0xffffff,
    metalness: options.metalness,
    roughness: options.roughness,
  });

  // Every X-Ray surface, because every one of them has a gloss and none of them has the reflectance three.js would
  // otherwise give it. First of the patches, so a later one can write the gloss it reads for itself.
  applyXrayGlossShading(material, options.gloss ?? XRAY_DEFAULT_GLOSS);

  applyRenderSurface(material, surface);

  return material;
}

/**
 * Puts a material into the state its surface's shader compiles to, leaving the recompile flag to the caller.
 *
 * @param material - Material being configured.
 * @param surface - Material state the surface's shader comes to.
 */
export function applyRenderSurface(material: MeshStandardMaterial, surface: IRenderSurface): void {
  material.alphaTest = surface.alphaTest;
  material.transparent = surface.isTransparent;
  material.depthWrite = surface.isDepthWritten;
  material.blending = surface.blend.blending;

  // A composited surface is a mark laid on another one - a stain, a crack, a poster - and a level author lays it in
  // the wall's own plane. Two triangles at one depth make the depth test a coin toss the camera flips on every step,
  // which reads as the mark flickering. Pulled towards the viewer rather than pushed back, so a mark is never the
  // thing that loses, and by the smallest slope-scaled amount that settles it.
  material.polygonOffset = surface.isTransparent;
  material.polygonOffsetFactor = DECAL_POLYGON_OFFSET;
  material.polygonOffsetUnits = DECAL_POLYGON_OFFSET;

  // Set unconditionally so a material re-dressed from a custom equation to a named one does not keep the old factors,
  // which three.js reads whenever `blending` is `CustomBlending` and ignores otherwise.
  material.blendSrc = surface.blend.blendSrc ?? SrcAlphaFactor;
  material.blendDst = surface.blend.blendDst ?? OneMinusSrcAlphaFactor;
}
