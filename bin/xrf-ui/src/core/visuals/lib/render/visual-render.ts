import { addVectors, normalise, scaleVector, toRadians } from "@xrf/math";
import { Nullable } from "@xrf/types";

import {
  ERenderCamera,
  ERenderOverlay,
  RenderCamera,
  RenderOverlay,
  RenderViewOptions,
} from "@/core/ipc/types/xrf-renderer";
import { IRenderLighting, toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import { toNativeLines } from "@/core/render/lib/native/native-overlay";
import { toNativeAssetViewOptions } from "@/core/render/lib/native/native-view-options";
import { toRawColor } from "@/core/render/lib/scene/render-color";
import { toRenderAxesLines, toRenderGridLines } from "@/core/render/lib/scene/render-grid-lines";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { IVisualPreviewSceneConfig } from "@/core/visuals/lib/scene/scene-config";
import { IVisualPreviewViewOptions } from "@/core/visuals/lib/scene/visual-view-options";
import { IVisualCameraFit } from "@/core/visuals/lib/visual-views";

/**
 * The camera a model is first framed from: back along the viewer's direction, far enough to fit its sphere.
 *
 * @param fit - The model's measured sphere.
 * @param config - The viewer's field of view, margin and direction.
 * @returns The orbit camera.
 */
export function toVisualCamera(fit: IVisualCameraFit, config: IVisualPreviewSceneConfig): RenderCamera {
  const { cameraFieldOfView, cameraFitMargin, cameraDirection } = config;
  const distance: number = (fit.radius / Math.sin(toRadians(cameraFieldOfView / 2))) * cameraFitMargin;

  return {
    far: distance * 100,
    fieldOfView: cameraFieldOfView,
    kind: ERenderCamera.ORBIT,
    near: Math.max(distance / 1000, 0.0001),
    position: addVectors(fit.center, scaleVector(normalise(cameraDirection), distance)),
    target: [...fit.center],
  };
}

/**
 * @param options - The toolbar's toggles.
 * @param lighting - The toolbar's light.
 * @param config - The viewer's backdrop and checker.
 * @param features - What the application sets every viewport's features to.
 * @param renderHeight - How many rows the model is drawn with at most; null for as many as the viewport covers.
 * @returns What a native viewport draws the model with.
 */
export function toVisualViewOptions(
  options: IVisualPreviewViewOptions,
  lighting: IRenderLighting,
  config: IVisualPreviewSceneConfig,
  features: IRenderFeatureSettings,
  renderHeight: Nullable<number> = null
): RenderViewOptions {
  return {
    ...toNativeAssetViewOptions(
      { isBumped: options.isBumpVisible, isLit: true, isWireframe: options.isWireframe },
      features,
      renderHeight
    ),
    assetLighting: toNativeAssetLighting(lighting),
    backdrop: toRawColor(config.backgroundColor),
    checker: options.isCheckerVisible ? config.checkerRepeat : 0,
    isAlphaVisible: options.isAlphaVisible,
    // Untextured, a surface is the viewer's plain mesh colour rather than white.
    plainColor: toRawColor(config.meshColor),
  };
}

/**
 * The grid, the axes, the skeleton and the marked joint, sized to the model and shown as the toolbar asks.
 *
 * @param options - The toolbar's toggles.
 * @param radius - How far the model reaches from its centre.
 * @param joint - The marked joint's position, or null for none.
 * @param config - The viewer's colours and sizes.
 * @returns The helpers a native viewport draws over the model.
 */
export function toVisualOverlays(
  options: Pick<IVisualPreviewViewOptions, "isAxesVisible" | "isGridVisible" | "isSkeletonVisible">,
  radius: number,
  joint: Nullable<[number, number, number]>,
  config: IVisualPreviewSceneConfig
): Array<RenderOverlay> {
  const overlays: Array<RenderOverlay> = [];

  if (options.isGridVisible) {
    overlays.push(
      toNativeLines(
        toRenderGridLines(radius, {
          cells: config.gridCells,
          color: config.gridColor,
          originColor: config.gridOriginColor,
        }),
        true
      )
    );
  }

  if (options.isAxesVisible) {
    overlays.push(toNativeLines(toRenderAxesLines(radius), true));
  }

  if (options.isSkeletonVisible) {
    overlays.push({ color: toRawColor(config.skeletonColor), isDepthTested: false, kind: ERenderOverlay.SKELETON });

    if (joint) {
      overlays.push({
        color: toRawColor(config.highlightColor),
        isDepthTested: false,
        kind: ERenderOverlay.POINTS,
        positions: [...joint],
        size: config.highlightSize,
      });
    }
  }

  return overlays;
}
