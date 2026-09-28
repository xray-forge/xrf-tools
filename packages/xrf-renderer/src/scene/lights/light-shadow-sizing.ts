import { ILightShadowAsk } from "#/scene/lights/light-shadow-ask";
import { LIGHT_SHADOW_ATLAS_SIZE } from "#/scene/lights/light-shadow-atlas";
import { LIGHT_SHADOW_POINT_CONE, toLightShadowFaceCount } from "#/scene/lights/light-shadow-faces";
import { ILightShadowRequest } from "#/scene/lights/light-shadow-request";

/** `SMAP_adapt_min`, `SMAP_adapt_optimal` and `SMAP_adapt_max` (`r2_types.h`). */
const SMAP_MIN: number = 32;
const SMAP_OPTIMAL: number = 768;
const SMAP_MAX: number = 1536;

/** The largest square a face takes, which leaves the atlas room for the rest. */
const TILE_MAX: number = 1024;

/** How far a face's wanted size may stray from the square it was asked at before it is asked at another. */
const GROW: number = 1.5;
const SHRINK: number = 0.66;

/** The share of the atlas the lights in view may want before every face is asked for smaller. */
const FILL: number = 0.75;

/** What the wanted sizes' scale is multiplied by for a frame a light in view was refused room, and by to recover. */
const TIGHTEN: number = 0.7;
const LOOSEN: number = 1.25;

/** The least the wanted sizes are scaled to. */
const MIN_SCALE: number = 1 / 32;

/** The texels of the whole atlas. */
const ATLAS_AREA: number = LIGHT_SHADOW_ATLAS_SIZE * LIGHT_SHADOW_ATLAS_SIZE;

/** The least square a face is given. */
export const LIGHT_SHADOW_MIN_SIZE: number = SMAP_MIN;

/**
 * `compute_xf_spot`'s map size: larger for a light nearer, brighter, facing the camera, longer and wider, in texels.
 *
 * @param request - The light.
 * @returns The size the engine would draw its map at.
 */
export function toLightShadowSize(request: ILightShadowRequest): number {
  const area: number = Math.min(Math.max((request.range * request.range) / (1 + request.distance ** 2), 0), 1);
  const cone: number = request.isSpot ? request.cone : LIGHT_SHADOW_POINT_CONE;
  const factor: number =
    Math.sqrt(area) *
    Math.pow(Math.max(request.intensity, 0), 1 / 16) *
    Math.pow(request.duel, 1 / 4) *
    Math.pow(request.range / 8, 1 / 4) *
    Math.sqrt(cone / (Math.PI / 2));

  return Math.min(Math.max(Math.floor(factor * SMAP_OPTIMAL), SMAP_MIN), SMAP_MAX);
}

/**
 * @param size - The size a face is wanted at.
 * @param current - The square it was asked at, or zero for none, kept while the size stays within `SHRINK..GROW` of it.
 * @param isTight - Whether the atlas is short of room, which takes the power of two at or below the size.
 * @returns The square it is to be asked at: the power of two nearest the size.
 */
export function toLightShadowTileSize(size: number, current: number, isTight: boolean = false): number {
  if (isTight) {
    return Math.min(Math.max(2 ** Math.floor(Math.log2(Math.max(size, 1))), SMAP_MIN), TILE_MAX);
  }

  if (current > 0 && size <= current * GROW && size >= current * SHRINK) {
    return current;
  }

  return Math.min(Math.max(2 ** Math.round(Math.log2(Math.max(size, 1))), SMAP_MIN), TILE_MAX);
}

/**
 * Scales the asks down by what a refusal says, or back up by a step once the asks at the next scale fit, measured
 * exactly as the asks are rounded, over the lights in view alone.
 *
 * @param scale - What every wanted size was scaled by this frame.
 * @param asks - The lights in view this frame.
 * @param refused - How many of them were given less than they asked.
 * @returns The scale for the next frame.
 */
export function fitLightShadowScale(scale: number, asks: ReadonlyArray<ILightShadowAsk>, refused: number): number {
  const room: number = FILL * ATLAS_AREA;

  if (refused > 0) {
    const estimate: number = Math.sqrt(room / Math.max(toDemand(asks, 1), 1));

    return Math.max(MIN_SCALE, Math.min(scale * TIGHTEN, estimate));
  }

  if (scale < 1) {
    const next: number = Math.min(1, scale * LOOSEN);

    return toDemand(asks, next) <= room ? next : scale;
  }

  return scale;
}

/** Texels the asks' faces take at a scale, each rounded as the scale rounds it. */
function toDemand(asks: ReadonlyArray<ILightShadowAsk>, scale: number): number {
  return asks.reduce((total: number, ask: ILightShadowAsk) => {
    const size: number = toLightShadowTileSize(ask.engineSize * scale, 0, scale < 1);

    return total + toLightShadowFaceCount(ask.isSpot) * size * size;
  }, 0);
}
