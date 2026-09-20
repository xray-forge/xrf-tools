import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";

/**
 * What a texture's body is lit by before anyone touches it.
 */
export const DEFAULT_TEXTURE_LIGHTING: IRenderLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 0.9,
  sunAzimuth: 45,
  sunColor: 0xffffff,
  sunElevation: 36,
  sunIntensity: 3.2,
};

/**
 * A flat fill and no direction at all, which is what the light switch turns the body over to.
 */
export const UNLIT_TEXTURE_LIGHTING: IRenderLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 1,
  sunColor: 0xffffff,
  sunAzimuth: 0,
  sunElevation: 90,
  sunIntensity: 0,
};

/** How far a drag across the whole viewport swings the light, in degrees. */
export const TEXTURE_LIGHT_DRAG_SPEED: number = 180;

/** Short of the poles, where a directional light stops telling a bumped surface from a flat one. */
export const TEXTURE_LIGHT_ELEVATION_LIMIT: number = 87;
