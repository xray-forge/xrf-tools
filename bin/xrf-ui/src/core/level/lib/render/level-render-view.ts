import {
  DEFAULT_RENDERER_GRASS_WIND,
  DEFAULT_RENDERER_LIGHTING,
  ERendererCameraController,
  ERendererDebugView,
  IRendererFeatureSettings,
  IRendererFlyCamera,
  IRendererLighting,
  IRendererSettings,
  IRenderFramePacing,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { LevelStart } from "@/core/ipc/types/xrf-app";
import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import {
  ILevelFeatureOptions,
  toLevelRendererAntialiasing,
  toLevelRendererFeature,
} from "@/core/level/lib/features/level-feature-options";
import { toLevelRendererFog } from "@/core/level/lib/lighting/level-fog";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toLevelRendererTreeWind } from "@/core/level/lib/lighting/level-wind";
import { ILevelLodOptions, toLevelRendererLod } from "@/core/level/lib/lod/level-lod-options";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { toRendererLighting } from "@/core/render/lib/lighting/render-lighting";

/**
 * Where a level opens, flown by the toolbar's speeds.
 *
 * @param bounds - The level's extent, or null for a level that reports none.
 * @param start - Where the backend opens the level, or null where it names nowhere.
 * @param options - The camera the toolbar asks for.
 * @param config - The near and far planes.
 * @returns The camera.
 */
export function toLevelCamera(
  bounds: Nullable<VisualBounds>,
  start: Nullable<LevelStart>,
  options: ILevelCameraOptions,
  config: ILevelRenderConfig
): IRendererFlyCamera {
  const { position, target }: ILevelViewpoint = toLevelStartViewpoint(bounds, start);

  return {
    boost: options.boost,
    far: config.cameraFar,
    fieldOfView: options.fieldOfView,
    kind: ERendererCameraController.FLY,
    near: config.cameraNear,
    position: [position.x, position.y, position.z],
    sensitivity: options.sensitivity,
    speed: options.speed,
    target: [target.x, target.y, target.z],
  };
}

/**
 * @param lighting - The level's light, fog and wind, as its controls set them.
 * @param isFogged - Whether the fog is drawn.
 * @param isWindy - Whether the trees and the grass sway: the trees by the wind's controls, the grass as the game's calm.
 * @param sky - The texture key of the sky cube the level is lit under, or null for the engine's noon sky.
 * @returns The engine's noon, pointed and scaled by those controls, under the level's sky.
 */
export function toLevelRendererLighting(
  lighting: ILevelLighting,
  isFogged: boolean,
  isWindy: boolean,
  sky: Nullable<string> = null
): IRendererLighting {
  return {
    ...toRendererLighting(lighting, DEFAULT_RENDERER_LIGHTING),
    fog: isFogged ? toLevelRendererFog(lighting) : null,
    grass: isWindy ? DEFAULT_RENDERER_GRASS_WIND : null,
    sky: sky ? { blend: 0, textures: [sky, sky] } : DEFAULT_RENDERER_LIGHTING.sky,
    trees: isWindy ? toLevelRendererTreeWind(lighting) : null,
  };
}

/** What a level view's renderer settings are made of. */
export interface ILevelRendererSettingsInputs {
  /** The toolbar's toggles. */
  options: ILevelViewOptions;
  /** The level's light, whose hemisphere strength the baked light toggle gates. */
  lighting: ILevelLighting;
  /** How far trees are drawn in full, which the impostors toggle gates. */
  lod: ILevelLodOptions;
  /** What the view sets over the settings' features for itself, which their toggles gate. */
  view: ILevelFeatureOptions;
  /** How the application paces a view's frames. */
  pacing: IRenderFramePacing;
  /** What the renderer's features are set to, which the level's toolbar narrows. */
  features: IRendererFeatureSettings;
  /** The backdrop. */
  config: ILevelRenderConfig;
}

/**
 * @param inputs - What the settings are made of.
 * @returns The renderer's settings.
 */
export function toLevelRendererSettings(inputs: ILevelRendererSettingsInputs): IRendererSettings {
  const { options, lighting, lod, view, pacing, features, config } = inputs;

  return {
    // Fogged, the renderer draws the sky as total fog itself; this shows only where there is none.
    backdrop: config.backgroundColor,
    debugView: ERendererDebugView.FINAL,
    features: {
      ...features,
      ambientOcclusion: toLevelRendererFeature("ambientOcclusion", features, view, options.isOccluded),
      antialiasing: toLevelRendererAntialiasing(features.antialiasing, view, options.isAntialiased),
      grass: toLevelRendererFeature("grass", features, view, options.isGrassy),
      // Timing every pass costs a level's frame rate a few percent, so only while the readout shows it.
      isGpuTimed: features.isGpuTimed && options.isAdvancedStatsVisible,
      isOcclusionCulled: features.isOcclusionCulled && options.isOcclusionCulled,
      lights: toLevelRendererFeature("lights", features, view, options.isLamplit),
      lod: toLevelRendererLod(features.lod, lod, options.isImpostors),
      shadows: toLevelRendererFeature("shadows", features, view, options.isShadowed),
      water: toLevelRendererFeature("water", features, view, options.isWaterVisible),
    },
    pacing,
    hemiStrength: options.isBaked ? lighting.hemiStrength : 0,
    isBumped: true,
    isLit: true,
    isWireframe: options.isWireframe,
    tonemapScale: 1,
  };
}
