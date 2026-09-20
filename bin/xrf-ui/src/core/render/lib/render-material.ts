import { MeshStandardMaterial } from "three";

import { IRenderSurface, OPAQUE_RENDER_SURFACE } from "@/core/render/lib/render-surface";

/** What a surface is made of, beyond what its shader says. X-Ray authors no metalness, so nothing here is guessed. */
export interface IRenderMaterialOptions {
  metalness: number;
  roughness: number;
  /** Tint under the base texture, white wherever one is bound or the surface would be dyed by it. */
  color?: number;
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
}
