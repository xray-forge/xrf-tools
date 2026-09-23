import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";

/**
 * How a level preview is lit.
 */
export interface ILevelLighting extends IRenderLighting {
  /** How much the baked hemisphere term darkens the ambient, `0` ignoring it and `1` applying it whole. */
  hemiStrength: number;
}

/**
 * The engine's noon as it stands: its own colours at their own strength, the baked hemisphere applied whole, and the
 * sun where `default_clear` puts it at twelve - `sun_altitude` -69 and `sun_longitude` -30, which `setHP` reads as a
 * heading and a pitch, so thirty degrees up at a bearing of -69.
 */
export const DEFAULT_LEVEL_LIGHTING: ILevelLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 1,
  hemiStrength: 1,
  sunAzimuth: -69,
  sunColor: 0xffffff,
  sunElevation: 30,
  sunIntensity: 1,
};
