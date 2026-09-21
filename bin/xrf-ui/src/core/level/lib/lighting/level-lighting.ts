import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";

/**
 * How a level preview is lit.
 */
export interface ILevelLighting extends IRenderLighting {
  /** How much the baked hemisphere term darkens the ambient, `0` ignoring it and `1` applying it whole. */
  hemiStrength: number;
}

/** Enough to read a level by before anything is touched, and close to what an overcast noon comes to. */
export const DEFAULT_LEVEL_LIGHTING: ILevelLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 1.1,
  hemiStrength: 0.25,
  sunAzimuth: 35,
  sunColor: 0xffffff,
  sunElevation: 55,
  sunIntensity: 1.8,
};
