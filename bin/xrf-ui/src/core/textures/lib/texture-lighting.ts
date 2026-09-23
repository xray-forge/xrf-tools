import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";

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

/** Short of the poles, where a directional light stops telling a bumped surface from a flat one. */
export const TEXTURE_LIGHT_ELEVATION_LIMIT: number = 87;

/**
 * Swings the light by a drag across the viewport.
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
  const elevation: number = lighting.sunElevation - (deltaY / (height || 1)) * TEXTURE_LIGHT_DRAG_SPEED;

  return {
    ...lighting,
    sunAzimuth: lighting.sunAzimuth + (deltaX / (width || 1)) * TEXTURE_LIGHT_DRAG_SPEED,
    sunElevation: Math.max(-TEXTURE_LIGHT_ELEVATION_LIMIT, Math.min(TEXTURE_LIGHT_ELEVATION_LIMIT, elevation)),
  };
}
