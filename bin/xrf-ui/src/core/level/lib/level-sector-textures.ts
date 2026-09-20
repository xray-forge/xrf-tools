import { ISectorInstanceViews, ISectorSectionViews, ISectorViews } from "@/core/level/lib/level-sector-views";
import { isAlphaRenderSurface } from "@/core/render/lib/render-surface";
import { Maybe } from "@/lib/types/general";

/**
 * One texture a sector needs, and what it has to survive upload with.
 */
export interface ISectorTextureRequest {
  /** The reference as the shader table spells it, which is what the set is keyed by. */
  reference: string;
  /** Whether any surface drawn with this file samples its alpha channel. */
  isAlphaRead: boolean;
}

/**
 * Every texture reference a sector names, base textures and lightmaps alike, from both kinds of surface.
 *
 * @param views - The sector.
 * @returns Its references, without repeats.
 */
export function listSectorTextures(views: ISectorViews): Array<ISectorTextureRequest> {
  const requests: Map<string, ISectorTextureRequest> = new Map();

  for (const { surface, render } of [...views.sections, ...views.instances]) {
    if (surface.textureName) {
      request(requests, surface.textureName, isAlphaRenderSurface(render));
    }

    // Only the one the renderer samples. The other half of the pair is R1's baked colour, and reading it was a
    // megabyte a lightmap for a texture nothing binds.
    if (surface.hemi) {
      request(requests, surface.hemi, false);
    }

    // Named by the surface's blender or by its base texture's descriptor rather than by the shader table, so a sector
    // that only fetched what its table names would draw its ground as the bare aerial photograph the base texture is.
    // Its alpha is never read: the modulation is a multiply of three channels.
    if (render.detail) {
      request(requests, render.detail.reference, false);
    }
  }

  return Array.from(requests.values());
}

/**
 * Whether any surface of a sector reads its texture's alpha channel.
 *
 * @param views - The sector.
 * @returns Whether anything in it is cut out or blended.
 */
export function hasAlphaSurfaces(views: ISectorViews): boolean {
  const drawn: Array<ISectorSectionViews | ISectorInstanceViews> = [...views.sections, ...views.instances];

  return drawn.some((it) => isAlphaRenderSurface(it.render));
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

/** Records one reference, keeping the alpha answer of whichever surface naming it needs it. */
function request(requests: Map<string, ISectorTextureRequest>, reference: string, isAlphaRead: boolean): void {
  const held: Maybe<ISectorTextureRequest> = requests.get(reference);

  if (held) {
    held.isAlphaRead ||= isAlphaRead;
  } else {
    requests.set(reference, { isAlphaRead, reference });
  }
}
