import {
  ERendererCameraController,
  ERendererDraw,
  IRendererFeatureSettings,
  IRendererGeometry,
  IRendererObject,
  IRendererOrbitCamera,
  IRendererSettings,
  IRendererSkeleton,
  IRendererSurface,
  IRenderFramePacing,
} from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { toRawColor } from "@/core/render/lib/scene/render-color";
import { toAssetRendererSettings } from "@/core/render/lib/settings/asset-renderer-settings";
import { IVisualPreviewSceneConfig } from "@/core/visuals/lib/scene/scene-config";
import { IVisualPreviewViewOptions } from "@/core/visuals/lib/scene/visual-view-options";
import { IVisualBumpFiles } from "@/core/visuals/lib/visual-bump";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import {
  getVisualSubmeshLevel,
  IVisualCameraFit,
  IVisualModelViews,
  IVisualSubmeshLevel,
  IVisualSubmeshViews,
} from "@/core/visuals/lib/visual-views";

/** Everything a model puts in the renderer, by the key it is held under. */
export const VISUAL_RENDER_KEYS = {
  axes: "axes",
  checker: "checker",
  grid: "grid",
  highlight: "highlight",
  motion: "motion",
  skeleton: "skeleton",
  skeletonOverlay: "skeleton-overlay",
  /** A submesh's geometry, surface and object share one key. */
  submesh: (index: number): string => `submesh:${index}`,
} as const;

/**
 * A submesh's geometry, copied: the views share one buffer, and a put hands its arrays over.
 *
 * @param submesh - The submesh's views over the packed buffer.
 * @returns Its geometry, one group, the authored tangent basis and the skin links where it carries them.
 */
export function toVisualGeometry(submesh: IVisualSubmeshViews): IRendererGeometry {
  return {
    binormal: submesh.binormals.slice(),
    clusters: submesh.clusters
      ? { ranges: submesh.clusters.ranges.slice(), spheres: submesh.clusters.spheres.slice() }
      : undefined,
    groups: [{ count: submesh.indices.length, slot: 0, start: 0 }],
    index: submesh.indices.slice(),
    normal: submesh.normals.slice(),
    position: submesh.positions.slice(),
    skinIndices: submesh.skinIndices?.slice(),
    skinWeights: submesh.skinWeights?.slice(),
    tangent: submesh.tangents.slice(),
    uv: submesh.uvs.slice(),
  };
}

/**
 * The surface a submesh draws with, as its shader resolved and as the toolbar asks.
 *
 * @param submesh - The submesh.
 * @param texture - The file its base was read from, or nothing yet.
 * @param bump - Its pair, or nothing.
 * @param options - The toolbar's toggles.
 * @param config - The viewer's colours and checker.
 * @returns The surface.
 */
export function toVisualSurface(
  submesh: IVisualSubmeshViews,
  texture: Nullable<IVisualTextureFile>,
  bump: Nullable<IVisualBumpFiles>,
  options: IVisualPreviewViewOptions,
  config: IVisualPreviewSceneConfig
): IRendererSurface {
  const base: Nullable<string> = options.isCheckerVisible ? VISUAL_RENDER_KEYS.checker : (texture?.logicalPath ?? null);
  // Alpha off draws every surface solid, for comparison with what its shader cuts or blends.
  const draw: ERendererDraw = options.isAlphaVisible ? submesh.surface.draw : ERendererDraw.OPAQUE;

  return {
    alphaReference: options.isAlphaVisible ? submesh.surface.alphaReference : undefined,
    // Untextured, a surface is the viewer's plain mesh colour rather than white.
    color: base ? undefined : toRawColor(config.meshColor),
    draw,
    isLit: submesh.surface.isLit,
    textures: {
      base: base ?? undefined,
      bump: bump?.bump.logicalPath,
      bumpCompanion: bump?.companion.logicalPath,
    },
    tiling: options.isCheckerVisible ? config.checkerRepeat : 1,
  };
}

/**
 * @param submesh - The submesh.
 * @param detail - How far down its collapse chain to draw: 0 is full detail, 1 the coarsest.
 * @param hasSkeleton - Whether the model has a skeleton for skinned submeshes to bind to.
 * @returns The object drawing it.
 */
export function toVisualObject(submesh: IVisualSubmeshViews, detail: number, hasSkeleton: boolean): IRendererObject {
  const key: string = VISUAL_RENDER_KEYS.submesh(submesh.index);
  const level: IVisualSubmeshLevel = getVisualSubmeshLevel(submesh, detail);

  return {
    drawRange: { count: level.count, start: level.start },
    geometry: key,
    skeleton: hasSkeleton && submesh.skinIndices ? VISUAL_RENDER_KEYS.skeleton : undefined,
    surfaces: [key],
  };
}

/**
 * @param views - The model.
 * @returns Its skeleton, copied, or null for a model with none.
 */
export function toVisualSkeleton(views: IVisualModelViews): Nullable<IRendererSkeleton> {
  return views.skeletonBinds
    ? { binds: views.skeletonBinds.slice(), pairs: views.skeletonPairs?.slice() ?? undefined }
    : null;
}

/**
 * The camera a model is first framed from: back along the viewer's direction, far enough to fit its sphere.
 *
 * @param fit - The model's measured sphere.
 * @param config - The viewer's field of view, margin and direction.
 * @returns The orbit camera.
 */
export function toVisualCamera(fit: IVisualCameraFit, config: IVisualPreviewSceneConfig): IRendererOrbitCamera {
  const { cameraFieldOfView, cameraFitMargin, cameraDirection } = config;
  const distance: number = (fit.radius / Math.sin((cameraFieldOfView * Math.PI) / 360)) * cameraFitMargin;
  const length: number = Math.hypot(...cameraDirection) || 1;
  const [x, y, z] = fit.center;

  return {
    far: distance * 100,
    fieldOfView: cameraFieldOfView,
    kind: ERendererCameraController.ORBIT,
    near: Math.max(distance / 1000, 0.0001),
    position: [
      x + (cameraDirection[0] / length) * distance,
      y + (cameraDirection[1] / length) * distance,
      z + (cameraDirection[2] / length) * distance,
    ],
    target: [x, y, z],
  };
}

/**
 * @param options - The toolbar's toggles.
 * @param config - The viewer's backdrop.
 * @param pacing - How the application paces a view's frames.
 * @param features - What the renderer's features are set to.
 * @returns The renderer's settings.
 */
export function toVisualRendererSettings(
  options: IVisualPreviewViewOptions,
  config: IVisualPreviewSceneConfig,
  pacing: IRenderFramePacing,
  features: IRendererFeatureSettings
): IRendererSettings {
  return toAssetRendererSettings(
    {
      backdrop: config.backgroundColor,
      isBumped: options.isBumpVisible,
      isLit: true,
      isWireframe: options.isWireframe,
      pacing,
    },
    features
  );
}
