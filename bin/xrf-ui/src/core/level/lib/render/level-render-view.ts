import {
  DEFAULT_RENDERER_LIGHTING,
  ERendererCameraController,
  ERendererDebugView,
  IRendererFlyCamera,
  IRendererLighting,
  IRendererSettings,
  TFrameRateLimit,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { toLevelRendererFog } from "@/core/level/lib/lighting/level-fog";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { toRendererLighting } from "@/core/render/lib/lighting/render-lighting";

/**
 * Where a level opens, flown by the toolbar's speeds.
 *
 * @param bounds - The level's extent, or null for a level that reports none.
 * @param options - The camera the toolbar asks for.
 * @param config - The near and far planes.
 * @returns The camera.
 */
export function toLevelCamera(
  bounds: Nullable<VisualBounds>,
  options: ILevelCameraOptions,
  config: ILevelRenderConfig
): IRendererFlyCamera {
  const { position, target }: ILevelViewpoint = toLevelStartViewpoint(bounds);

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
 * @param lighting - The level's light and fog, as its controls set them.
 * @param isFogged - Whether the fog is drawn.
 * @returns The engine's noon, pointed and scaled by those controls.
 */
export function toLevelRendererLighting(lighting: ILevelLighting, isFogged: boolean): IRendererLighting {
  return {
    ...toRendererLighting(lighting, DEFAULT_RENDERER_LIGHTING),
    fog: isFogged ? toLevelRendererFog(lighting) : null,
  };
}

/**
 * @param options - The toolbar's toggles.
 * @param lighting - The level's light, whose hemisphere strength the baked light toggle gates.
 * @param frameRateLimit - How often the application lets a view redraw.
 * @param config - The backdrop.
 * @returns The renderer's settings.
 */
export function toLevelRendererSettings(
  options: ILevelViewOptions,
  lighting: ILevelLighting,
  frameRateLimit: TFrameRateLimit,
  config: ILevelRenderConfig
): IRendererSettings {
  return {
    // Fogged, the renderer draws the sky as total fog itself; this shows only where there is none.
    backdrop: config.backgroundColor,
    debugView: ERendererDebugView.FINAL,
    frameRateLimit,
    hemiStrength: options.isLit ? lighting.hemiStrength : 0,
    isBumped: true,
    isLit: true,
    isWireframe: options.isWireframe,
    tonemapScale: 1,
  };
}
