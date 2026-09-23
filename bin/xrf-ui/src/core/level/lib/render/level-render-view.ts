import {
  DEFAULT_RENDERER_LIGHTING,
  ERendererCameraController,
  ERendererDebugView,
  IRendererFlyCamera,
  IRendererFog,
  IRendererLighting,
  IRendererSettings,
  TFrameRateLimit,
  toneMapReinhard,
  TRendererColor,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";
import { ILevelCameraOptions } from "@/core/level/lib/camera/level-camera-options";
import { ILevelViewpoint, toLevelStartViewpoint } from "@/core/level/lib/camera/level-viewpoint";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { ILevelRenderConfig } from "@/core/level/lib/render/level-render-config";
import { ILevelViewOptions } from "@/core/level/lib/view/level-view-options";
import { toRendererLighting } from "@/core/render/lib/lighting/render-lighting";

/**
 * `default_clear`'s noon fog (`configs/environment/weathers/default_clear.ltx`, `[12:00:00]`): starts at a tenth of
 * 85% of its distance and is total at the far plane, 350 metres.
 */
export const LEVEL_NOON_FOG: IRendererFog = {
  color: [0.304609, 0.328138, 0.367354],
  density: 0.9,
  distance: 350,
};

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
 * @param lighting - The level's light, as its controls set it.
 * @param isFogged - Whether the noon fog is drawn.
 * @returns The engine's noon, pointed and scaled by those controls.
 */
export function toLevelRendererLighting(lighting: ILevelLighting, isFogged: boolean): IRendererLighting {
  return { ...toRendererLighting(lighting, DEFAULT_RENDERER_LIGHTING), fog: isFogged ? LEVEL_NOON_FOG : null };
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
    // Fogged, the sky is what total fog comes to, so the horizon does not end in a line.
    backdrop: options.isFogged ? toToneMappedHex(LEVEL_NOON_FOG.color, 1) : config.backgroundColor,
    debugView: ERendererDebugView.FINAL,
    frameRateLimit,
    hemiStrength: options.isLit ? lighting.hemiStrength : 0,
    isBumped: true,
    isLit: true,
    isWireframe: options.isWireframe,
    tonemapScale: 1,
  };
}

/** A raw colour through the engine's tonemap, as the hex a page writes. */
function toToneMappedHex(color: TRendererColor, scale: number): number {
  const [red, green, blue] = color.map((channel: number) =>
    Math.round(Math.min(1, toneMapReinhard(channel, scale)) * 255)
  );

  return (red << 16) | (green << 8) | blue;
}
