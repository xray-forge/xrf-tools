import { ERendererCameraController, ERendererDebugView, IRendererFlyCamera, IRendererSettings } from "@xrf/renderer";

import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import {
  ILevelFeatureOptions,
  toLevelRendererAntialiasing,
  toLevelRendererFeature,
} from "@/core/level/lib/features/level-feature-options";
import { ILevelLodOptions, toLevelRendererLod } from "@/core/level/lib/lod/level-lod-options";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { IRenderSharedSettings } from "@/core/render/lib/settings/render-shared-settings";

/**
 * @param viewpoint - Where the camera stands and what it looks at.
 * @param options - The camera the toolbar asks for.
 * @param config - The near and far planes.
 * @returns The camera there, flown by the toolbar's speeds.
 */
export function toLevelCameraAt(
  viewpoint: ILevelViewpoint,
  options: ILevelCameraOptions,
  config: ILevelRenderConfig
): IRendererFlyCamera {
  const { position, target }: ILevelViewpoint = viewpoint;

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

/** What a level view's renderer settings are made of. */
export interface ILevelRendererSettingsInputs {
  /** The toolbar's toggles. */
  options: ILevelViewOptions;
  /** How much the baked hemisphere darkens the ambient, which the baked light toggle gates. */
  hemiStrength: number;
  /** How far trees are drawn in full, which the impostors toggle gates. */
  lod: ILevelLodOptions;
  /** What the view sets over the settings' features for itself, which their toggles gate. */
  view: ILevelFeatureOptions;
  /** What the application sets for every viewport, whose features the level's toolbar narrows. */
  shared: IRenderSharedSettings;
  /** The backdrop. */
  config: ILevelRenderConfig;
}

/**
 * @param inputs - What the settings are made of.
 * @returns The renderer's settings.
 */
export function toLevelRendererSettings(inputs: ILevelRendererSettingsInputs): IRendererSettings {
  const { options, hemiStrength, lod, view, shared, config } = inputs;
  const { features } = shared;

  return {
    // Fogged, the renderer draws the sky as total fog itself; this shows only where there is none.
    backdrop: config.backgroundColor,
    debugView: ERendererDebugView.FINAL,
    features: {
      ...features,
      ambientOcclusion: toLevelRendererFeature("ambientOcclusion", features, view, options.isOccluded),
      antialiasing: toLevelRendererAntialiasing(features.antialiasing, view, options.isAntialiased),
      grass: toLevelRendererFeature("grass", features, view, options.isGrassy),
      isOcclusionCulled: features.isOcclusionCulled && options.isOcclusionCulled,
      lights: toLevelRendererFeature("lights", features, view, options.isLamplit),
      lod: toLevelRendererLod(features.lod, lod, options.isImpostors),
      shadows: toLevelRendererFeature("shadows", features, view, options.isShadowed),
      water: toLevelRendererFeature("water", features, view, options.isWaterVisible),
    },
    hemiStrength: options.isBaked ? hemiStrength : 0,
    isBumped: options.isBumped,
    isGpuTimed: shared.isGpuTimed,
    isLit: true,
    isSkyDrawn: options.isSkyVisible,
    isTextured: options.isTextured,
    isWireframe: options.isWireframe,
    pacing: shared.pacing,
    tonemapScale: 1,
  };
}
