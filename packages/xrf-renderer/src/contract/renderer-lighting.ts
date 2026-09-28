import { Nullable } from "@xrf/types";

import { TRendererColor } from "#/contract/renderer-color";
import { IRendererFog } from "#/contract/renderer-fog";
import { IRendererGrassWind } from "#/contract/renderer-grass-wind";
import { IRendererSky } from "#/contract/renderer-sky";
import { IRendererTreeWind } from "#/contract/renderer-tree-wind";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * What a scene is lit by, in the terms a weather keyframe uses.
 */
export interface IRendererLighting {
  /** The direction sunlight travels, in renderer space. */
  sunDirection: TRendererVector;
  /** `sun_color`. */
  sunColor: TRendererColor;
  /** `hemisphere_color`. */
  hemisphereColor: TRendererColor;
  /** `ambient_color`. */
  ambientColor: TRendererColor;
  /** What the sky's irradiance cube returns, standing in for one until a weather supplies it. */
  skyIrradiance: TRendererColor;
  /** Distance fog, or none. */
  fog: Nullable<IRendererFog>;
  /** How the trees sway, or null for trees standing still. */
  trees: Nullable<IRendererTreeWind>;
  /** How the grass sways, or null for grass standing still. */
  grass: Nullable<IRendererGrassWind>;
  /** The sky the water reflects. */
  sky: IRendererSky;
  /** `water_intensity`: how bright the depth of soft water and its foam are, one by a clear day. */
  waterIntensity: number;
}
