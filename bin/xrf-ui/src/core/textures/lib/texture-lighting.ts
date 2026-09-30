import { clamp, wrap } from "@xrf/math";

import { IRenderLighting, RENDER_LIGHTING_LIMITS } from "@/core/render/lib/lighting/render-lighting";

/**
 * What a texture's body is lit by before anyone touches it: the engine's noon, from the upper right.
 */
export const DEFAULT_TEXTURE_LIGHTING: IRenderLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 1,
  sunAzimuth: 45,
  sunColor: 0xffffff,
  sunElevation: 36,
  sunIntensity: 1,
};

/** How far a drag across the whole viewport swings the light, in degrees. */
export const TEXTURE_LIGHT_DRAG_SPEED: number = 180;

/** Short of the zenith, where a directional light stops telling a bumped surface from a flat one. */
export const TEXTURE_LIGHT_ELEVATION_LIMIT: number = 87;

/**
 * Swings the light by a drag across the viewport, within what the sliders offer: the azimuth wrapped round, the
 * elevation held between the sliders' floor and short of the zenith.
 *
 * @param lighting - Where the light is.
 * @param deltaX - Horizontal movement in pixels.
 * @param deltaY - Vertical movement in pixels.
 * @param width - Viewport width in pixels.
 * @param height - Viewport height in pixels.
 * @returns Where the drag has put the light.
 */
export function dragTextureLighting(
  lighting: IRenderLighting,
  deltaX: number,
  deltaY: number,
  width: number,
  height: number
): IRenderLighting {
  const { sunAzimuth, sunElevation } = RENDER_LIGHTING_LIMITS;
  const azimuth: number = lighting.sunAzimuth + (deltaX / (width || 1)) * TEXTURE_LIGHT_DRAG_SPEED;
  const elevation: number = lighting.sunElevation - (deltaY / (height || 1)) * TEXTURE_LIGHT_DRAG_SPEED;
  const turn: number = sunAzimuth.max - sunAzimuth.min;

  return {
    ...lighting,
    sunAzimuth: wrap(azimuth - sunAzimuth.min, turn) + sunAzimuth.min,
    sunElevation: clamp(elevation, sunElevation.min, TEXTURE_LIGHT_ELEVATION_LIMIT),
  };
}
