import { ERendererLightShadowFilter } from "@/core/render/lib/contract/renderer-light-shadow-filter";

/**
 * The local lights a scene was given: binned into a grid over the view, and accumulated after the sun in one pass.
 */
export interface IRendererLightsSettings {
  isEnabled: boolean;
  /** Whether the level file's own lights are drawn too, which the engine does only with `r2_allow_r1_lights`. */
  isLevelLights: boolean;
  /** Whether a light the engine shadows casts its shadows, through faces drawn once into an atlas and kept. */
  isShadowed: boolean;
  shadowFilter: ERendererLightShadowFilter;
}

/** The engine's own: every spawned light, and none of the level file's. */
export const DEFAULT_RENDERER_LIGHTS_SETTINGS: IRendererLightsSettings = {
  isEnabled: true,
  isLevelLights: false,
  isShadowed: true,
  shadowFilter: ERendererLightShadowFilter.ENGINE,
};
