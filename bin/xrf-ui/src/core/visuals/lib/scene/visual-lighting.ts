import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";

/**
 * What a model is lit by before anyone touches it.
 *
 * Bright enough to read an X-Ray texture, which is the whole reason these numbers are what they are: the game's
 * albedo is dark by design - mud, rusted steel, wet coats - and a physically modest light over it leaves a model that
 * can be made out rather than looked at. The angle is off axis in both directions so a surface's shape reads.
 */
export const DEFAULT_VISUAL_LIGHTING: IRenderLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 2.2,
  sunAzimuth: 37,
  sunColor: 0xffffff,
  sunElevation: 45,
  sunIntensity: 3,
};
