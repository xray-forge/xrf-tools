import { IRendererGrassWind } from "#/contract/renderer-grass-wind";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererTreeWind } from "#/contract/renderer-tree-wind";
import { toRendererSunDirection } from "#/lighting/sun-direction";

/** The engine's own sway, where a weather states none (`CEnvDescriptor::load`). */
export const DEFAULT_RENDERER_TREE_WIND: IRendererTreeWind = {
  amplitude: 0.005,
  rotation: 10,
  speed: 1,
  wave: [0.1, 0.01, 0.11],
};

/**
 * The engine's grass swing (`system.ltx` `[details]`), at the strength a still level gives it: with no gusts the
 * weather's Perlin noise is zero, so `wind_strength_factor` is a half.
 */
export const DEFAULT_RENDERER_GRASS_WIND: IRendererGrassWind = {
  fast: { amp1: 0.35, amp2: 0.2, rot1: 5, rot2: 0.5, speed: 0.5 },
  normal: { amp1: 0.1, amp2: 0.05, rot1: 30, rot2: 1, speed: 2 },
  strength: 0.5,
};

/** The sky `default_clear` names at noon, both keyframes' at exactly twelve: the next one is weighed at nothing. */
const DEFAULT_RENDERER_SKY_TEXTURE: string = "sky\\sky_7_cube";

/** Its irradiance cube, which lights the hemisphere. */
const DEFAULT_RENDERER_SKY_ENVIRONMENT: string = "sky\\sky_7_cube#small";

/**
 * Noon of `default_clear` (`configs/environment/weathers/default_clear.ltx`, `[12:00:00]`), without its fog.
 * The sky irradiance is the mean of that keyframe's `sky_7_cube#small`, measured at 0.50, 0.51 and 0.55.
 */
export const DEFAULT_RENDERER_LIGHTING: IRendererLighting = {
  ambientColor: [0.02, 0.02, 0.02],
  fog: null,
  hemisphereColor: [0.470588, 0.368627, 0.329412],
  isExtendedShading: false,
  rain: null,
  skyIrradiance: [0.5, 0.511, 0.548],
  sunColor: [0.905882, 0.839216, 0.694118],
  grass: DEFAULT_RENDERER_GRASS_WIND,
  sky: {
    blend: 0,
    clouds: null,
    color: [0.851001, 0.851001, 0.851001],
    environments: [DEFAULT_RENDERER_SKY_ENVIRONMENT, DEFAULT_RENDERER_SKY_ENVIRONMENT],
    isCurved: false,
    rotation: 0,
    textures: [DEFAULT_RENDERER_SKY_TEXTURE, DEFAULT_RENDERER_SKY_TEXTURE],
  },
  sunDirection: toRendererSunDirection(-68.999985, -30),
  thunderbolt: null,
  trees: DEFAULT_RENDERER_TREE_WIND,
  waterIntensity: 1,
};
