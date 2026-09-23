import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { SectorDescription, SectorSurface } from "@/core/ipc/types/xrf-visual";
import { ISectorInstanceViews, ISectorSectionViews, ISectorViews } from "@/core/level/lib/sector/level-sector-views";
import { getLevelSurfaceRender, ILevelSurfaceRender } from "@/core/level/lib/surface/level-surface-render";

/**
 * One texture a sector needs.
 */
export interface ISectorTextureRequest {
  /** The reference as the shader table spells it, which is what the renderer holds it under. */
  reference: string;
}

/** A surface a sector draws, with what its blender compiles to. */
interface ISectorDrawnSurface {
  surface: SectorSurface;
  render: ILevelSurfaceRender;
}

/**
 * Every texture reference a sector names, base textures and lightmaps alike, from both kinds of surface.
 *
 * @param views - The sector.
 * @returns Its references, without repeats.
 */
export function listSectorTextures(views: ISectorViews): Array<ISectorTextureRequest> {
  const references: Set<string> = new Set();

  for (const drawn of [...views.sections, ...views.instances]) {
    collectSurfaceTextures(references, drawn);
  }

  return toRequests(references);
}

/**
 * Every texture reference a sector's pack names, read from the description alone.
 *
 * @param description - What `open_sector` reported about the pack.
 * @param surfaces - The level's resolved shader table.
 * @returns Its references, without repeats.
 */
export function listDescriptionTextures(
  description: SectorDescription,
  surfaces: ReadonlyArray<XraySurfaceDescriptor>
): Array<ISectorTextureRequest> {
  const references: Set<string> = new Set();

  for (const drawn of [...description.sections, ...description.instances]) {
    collectSurfaceTextures(references, {
      render: getLevelSurfaceRender(surfaces, drawn.surface.shaderId),
      surface: drawn.surface,
    });
  }

  return toRequests(references);
}

/** What one surface names, folded into whatever the caller is collecting. */
function collectSurfaceTextures(references: Set<string>, { surface, render }: ISectorDrawnSurface): void {
  if (surface.textureName) {
    references.add(surface.textureName);
  }

  // Only the one the renderer samples. The other half of the pair is R1's baked colour, and reading it was a
  // megabyte a lightmap for a texture nothing binds.
  if (surface.hemi) {
    references.add(surface.hemi);
  }

  // Named by the surface's blender or by its base texture's descriptor rather than by the shader table, so a sector
  // that only fetched what its table names would draw its ground as the bare aerial photograph the base texture is.
  if (render.detail) {
    references.add(render.detail.reference);
  }
}

/**
 * Whether any surface of a sector is modulated by a detail texture.
 *
 * @param views - The sector.
 * @returns Whether anything in it carries the high frequency half of an X-Ray surface.
 */
export function hasDetailedSurfaces(views: ISectorViews): boolean {
  const drawn: Array<ISectorSectionViews | ISectorInstanceViews> = [...views.sections, ...views.instances];

  return drawn.some((it) => Boolean(it.render.detail));
}

function toRequests(references: ReadonlySet<string>): Array<ISectorTextureRequest> {
  return Array.from(references, (reference: string) => ({ reference }));
}
