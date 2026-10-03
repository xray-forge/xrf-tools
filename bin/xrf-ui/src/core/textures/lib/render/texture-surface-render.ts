import { Nullable } from "@xrf/types";

import { TextureDescription } from "@/core/ipc/types/xrf-app";
import { ERenderCamera, RenderCamera, RenderViewOptions } from "@/core/ipc/types/xrf-renderer";
import { IRenderLighting, toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import {
  NEUTRAL_NATIVE_LOOK,
  NO_NATIVE_VIEW_SWITCHES,
  toNativeViewOptions,
} from "@/core/render/lib/native/native-view-options";
import { TNativeTextureRequest } from "@/core/render/lib/native/native-viewport";
import { toRawColor } from "@/core/render/lib/scene/render-color";
import { toAssetRendererSettings } from "@/core/render/lib/settings/asset-renderer-settings";
import { IRenderSharedSettings } from "@/core/render/lib/settings/render-shared-settings";
import { ITextureSurfaceOptions, toTextureAspect } from "@/core/textures/lib/texture-surface";
import { VIEWPORT } from "@/core/theme/tokens";

/** The camera the body is first seen from, and returned to. */
export const TEXTURE_SURFACE_CAMERA: RenderCamera = {
  far: 100,
  fieldOfView: 45,
  kind: ERenderCamera.ORBIT,
  near: 0.01,
  position: [0, 0, 5],
  target: [0, 0, 0],
};

/** The slab's four edges and its back: dark and plain, so a turned slab reads as a slab rather than as the texture. */
const TEXTURE_EDGE_COLOR: number = 0x1a1a1a;

/**
 * @param color - A colour as css writes it, `#rrggbb`.
 * @returns The same as one number.
 */
function toHexColor(color: string): number {
  return Number.parseInt(color.slice(1), 16);
}

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
 * @param shared - What the application sets for every viewport.
 * @param pixelRatio - Device pixels a css pixel, which the backdrop's squares are measured in.
 * @param renderHeight - How many rows the body is drawn with at most; null for as many as the viewport covers.
 * @returns What a native viewport draws the body with: against the alpha checkerboard every picture is judged on.
 */
export function toTextureViewOptions(
  options: ITextureSurfaceOptions,
  lighting: IRenderLighting,
  shared: IRenderSharedSettings,
  pixelRatio: number,
  renderHeight: Nullable<number> = null
): RenderViewOptions {
  const settings = toAssetRendererSettings(
    { backdrop: null, isBumped: options.isBumped, isLit: options.isLit, isWireframe: false },
    shared
  );

  return {
    ...toNativeViewOptions(
      settings,
      NO_NATIVE_VIEW_SWITCHES,
      { ...NEUTRAL_NATIVE_LOOK, exposure: settings.features.exposure },
      renderHeight
    ),
    assetLighting: toNativeAssetLighting(lighting),
    backdrop: toRawColor(toHexColor(VIEWPORT.checkerboardDark)),
    backdropSquares: {
      color: toRawColor(toHexColor(VIEWPORT.checkerboardLight)),
      size: VIEWPORT.checkerboardSquare * pixelRatio,
    },
    plainColor: toRawColor(TEXTURE_EDGE_COLOR),
  };
}
