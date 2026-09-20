/**
 * How a level preview is lit.
 */
export interface ILevelLighting {
  /** Degrees above the horizon the sun sits at, `90` being directly overhead. */
  sunElevation: number;
  /** Degrees around the vertical axis, `0` looking along the level's own `+z`. */
  sunAzimuth: number;
  sunIntensity: number;
  /** Hex colour of the directional light. */
  sunColor: number;
  /** Uniform light standing in for the hemisphere the deferred pass samples. */
  ambientIntensity: number;
  ambientColor: number;
  /** How much the baked hemisphere term darkens the ambient, `0` ignoring it and `1` applying it whole. */
  hemiStrength: number;
}

/** Enough to read a level by before anything is touched, and close to what an overcast noon comes to. */
export const DEFAULT_LEVEL_LIGHTING: ILevelLighting = {
  ambientColor: 0xffffff,
  ambientIntensity: 1.1,
  hemiStrength: 0.65,
  sunAzimuth: 35,
  sunColor: 0xffffff,
  sunElevation: 55,
  sunIntensity: 1.8,
};
