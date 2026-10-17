import { parseHexColor, toRawColor } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { RenderViewOptions } from "@/core/ipc/types/xrf-renderer";
import { EWorldCamera, WorldCamera } from "@/core/ipc/types/xrf-world";
import { IRenderLighting, toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import { toNativeAssetViewOptions } from "@/core/render/lib/native/native-view-options";
import { TNativeTextureRequest } from "@/core/render/lib/native/native-viewport";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { NATIVE_RENDER_OUTPUT, TRenderOutput } from "@/core/render/lib/settings/render-resolution";
import { ITextureSurfaceOptions, toTextureAspect } from "@/core/textures/lib/texture-surface";
import { VIEWPORT } from "@/core/theme/tokens";

/** The camera the body is first seen from, and returned to. */
export const TEXTURE_SURFACE_CAMERA: WorldCamera = {
  far: 100,
  fieldOfView: 45,
  kind: EWorldCamera.ORBIT,
  near: 0.01,
  position: [0, 0, 5],
  target: [0, 0, 0],
};

/**
 * @param description - The texture on screen, or null for none.
 * @param options - How it is being looked at.
 * @returns What the renderer is asked to draw: the texture on its body, or nothing.
 */
export function toTextureSurfaceRequest(
  description: Nullable<TextureDescription>,
  options: ITextureSurfaceOptions
): TNativeTextureRequest {
  return description
    ? {
        alpha: options.alpha,
        aspect: toTextureAspect(description),
        roots: description.roots,
        shape: options.shape,
        source: description.source,
        tiling: options.tiling,
      }
    : null;
}

/**
 * @param options - How the texture is being looked at.
 * @param lighting - The light the body is shaded by.
 * @param features - What the application sets every viewport's features to.
 * @param pixelRatio - Device pixels a css pixel, which the backdrop's squares are measured in.
 * @param output - What the body's frame is resolved at before the viewport.
 * @returns What a native viewport draws the body with: against the alpha checkerboard every picture is judged on.
 */
export function toTextureViewOptions(
  options: ITextureSurfaceOptions,
  lighting: IRenderLighting,
  features: IRenderFeatureSettings,
  pixelRatio: number,
  output: TRenderOutput = NATIVE_RENDER_OUTPUT
): RenderViewOptions {
  const view: RenderViewOptions = toNativeAssetViewOptions(
    { isBumped: options.isBumped, isLit: options.isLit, isWireframe: false },
    features,
    output
  );

  return {
    ...view,
    asset: {
      ...view.asset,
      backdrop: toRawColor(parseHexColor(VIEWPORT.checkerboardDark)),
      backdropSquares: {
        color: toRawColor(parseHexColor(VIEWPORT.checkerboardLight)),
        size: VIEWPORT.checkerboardSquare * pixelRatio,
      },
      lighting: toNativeAssetLighting(lighting),
      plainColor: toRawColor(parseHexColor(VIEWPORT.slabEdge)),
    },
  };
}
