import { Nullable } from "@xrf/types";

import { RenderViewOptions } from "@/core/ipc/types/xrf-renderer";
import { EWorldCamera, WorldCamera } from "@/core/ipc/types/xrf-world";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import {
  ILevelFeatureOptions,
  toLevelIndirectLight,
  toLevelRain,
  toLevelReflections,
  toLevelRendererAntialiasing,
  toLevelRendererFeature,
} from "@/core/level/lib/features/level-feature-options";
import { ILevelLodOptions, toLevelRendererLod } from "@/core/level/lib/lod/level-lod-options";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ELevelShading, getLevelShading, ILevelShadingChoice } from "@/core/level/lib/view/level-shading";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { ILevelSunShaftsOptions } from "@/core/level/lib/weather/level-sun-shafts-options";
import { TNativeLook, toNativeViewOptions } from "@/core/render/lib/native/native-view-options";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";

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
): WorldCamera {
  const { position, target }: ILevelViewpoint = viewpoint;

  return {
    boost: options.boost,
    far: config.cameraFar,
    fieldOfView: options.fieldOfView,
    kind: EWorldCamera.FLY,
    near: config.cameraNear,
    position: [position.x, position.y, position.z],
    sensitivity: options.sensitivity,
    speed: options.speed,
    target: [target.x, target.y, target.z],
  };
}

/** What a level view's options are made of. */
export interface ILevelViewOptionsInputs {
  /** The toolbar's toggles, which switch the scene's weather, sky, grass, water and spawned groups too. */
  options: ILevelViewOptions;
  /** How much the baked hemisphere darkens the ambient, which the baked light toggle gates. */
  hemiStrength: number;
  /** How finely the sunshafts step, and Monolith's floor under their density, which their toggle gates. */
  sunShafts: ILevelSunShaftsOptions;
  /** How far trees are drawn in full, which the impostors toggle gates. */
  lod: ILevelLodOptions;
  /** What the view sets over the settings' features for itself, which their toggles gate. */
  view: ILevelFeatureOptions;
  /** What the application sets every viewport's features to, which the level's toolbar narrows. */
  features: IRenderFeatureSettings;
  /** How the level is exposed, lit and corrected. */
  look: TNativeLook;
  /** What the viewport shows of its surfaces. */
  shading: ELevelShading;
  /** How many rows the level is drawn with at most; null for as many as the viewport covers. */
  renderHeight: Nullable<number>;
}

/**
 * @param inputs - What the options are made of.
 * @returns What the native viewport draws the level with.
 */
export function toLevelViewOptions(inputs: ILevelViewOptionsInputs): RenderViewOptions {
  const { options, hemiStrength, sunShafts, lod, view, features, look, shading, renderHeight } = inputs;
  const { debugView, surfaceColor }: ILevelShadingChoice = getLevelShading(shading);

  return toNativeViewOptions(
    {
      debugView,
      hemiStrength: options.isBaked ? hemiStrength : 0,
      isBumped: options.isBumped,
      isLit: true,
      isWireframe: options.isWireframe,
      surfaceColor,
    },
    {
      ...features,
      ambientOcclusion: toLevelRendererFeature("ambientOcclusion", features, view, options.isOccluded),
      antialiasing: toLevelRendererAntialiasing(features.antialiasing, view, options.isAntialiased),
      grass: toLevelRendererFeature("grass", features, view, options.isGrassy),
      indirectLight: toLevelIndirectLight(features, view),
      isOcclusionCulled: features.isOcclusionCulled && options.isOcclusionCulled,
      lights: toLevelRendererFeature("lights", features, view, options.isLamplit),
      lod: toLevelRendererLod(features.lod, lod, options.isImpostors),
      rain: toLevelRain(features, view),
      reflections: toLevelReflections(features, view),
      shadows: toLevelRendererFeature("shadows", features, view, options.isShadowed),
      water: toLevelRendererFeature("water", features, view, options.isWaterVisible),
    },
    { ...options, sunShafts },
    look,
    renderHeight
  );
}
