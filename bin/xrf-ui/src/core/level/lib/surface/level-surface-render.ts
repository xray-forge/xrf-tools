import { Maybe, Nullable } from "@xrf/types";

import { EXraySurfaceDraw, XraySurfaceDescriptor, XraySurfaceSampler } from "@/core/ipc/types/xrf-material";
import { ERenderPass, toRenderPass } from "@/core/render/lib/surface/render-pass";
import {
  IRenderSurfaceDraw,
  isWallmarkSurface,
  toRenderSurfaceDraw,
} from "@/core/render/lib/surface/render-surface-draw";

/**
 * The detail texture a surface modulates its base with, as the shader table's answer named it.
 */
export interface ILevelSurfaceDetail {
  /** Texture reference, engine-style. */
  reference: string;
  /** Times it repeats across the surface's base coordinate. */
  scale: number;
  /** The detail's own bump pair, added to the surface's, or null where the surface binds none of it. */
  bump: Nullable<ILevelSurfaceBump>;
}

/**
 * The bump pair a surface binds, as its base texture's descriptor declared it: each the reference the engine asks for,
 * which the level found as the file it names or the engine's dummy in its place.
 */
export interface ILevelSurfaceBump {
  /** `normal.gloss`. */
  bump: string;
  /** `normal_error.height`, the bump's name with `#` appended. */
  companion: string;
}

/**
 * The textures a water surface's script binds by sampler, each a reference, engine-style, or null where it binds none.
 */
export interface ILevelSurfaceWaterTextures {
  /** `s_base`, the water's own, which the script names in place of the level's row. */
  base: Nullable<string>;
  /** `s_nmap`. */
  normal: Nullable<string>;
  /** `s_leaves`. */
  foam: Nullable<string>;
  /** `s_distort`, which its distortion element binds. */
  distortion: Nullable<string>;
}

/**
 * What one shader table entry compiles to, for the renderer: its draw, whether it is a wall mark, and its detail.
 */
export interface ILevelSurfaceRender extends IRenderSurfaceDraw {
  /** Whether a scripted pass says it is a wall mark, which the renderer composites into the albedo before light. */
  isWallmark: boolean;
  /** The detail bound beside the base, or null for a surface the engine details with none. */
  detail: Nullable<ILevelSurfaceDetail>;
  /** The bump pair bound beside the base, or null for a surface the engine draws flat. */
  bump: Nullable<ILevelSurfaceBump>;
  /** The lighting model its base texture's descriptor sets: its class plus its weight. */
  material: number;
  /** What a water surface's script binds, or null for any other surface. */
  waterTextures: Nullable<ILevelSurfaceWaterTextures>;
}

/** The function whose pass is the surface itself, and the one drawing its distortion. */
/** The texture descriptor's default lighting model: Blinn, at full weight (`SH_Texture.cpp`). */
const DEFAULT_MATERIAL: number = 1;

const BASE_ELEMENT: string = "normal";
const DISTORTION_ELEMENT: string = "l_special";

/**
 * @param samplers - What a surface's script binds.
 * @param element - The function whose pass binds it.
 * @param name - The sampler.
 * @returns The texture it binds there, or null for none.
 */
function findSamplerTexture(
  samplers: ReadonlyArray<XraySurfaceSampler>,
  element: string,
  name: string
): Nullable<string> {
  return samplers.find((it: XraySurfaceSampler) => it.element === element && it.name === name)?.texture ?? null;
}

/**
 * @param descriptor - What the backend resolved for the surface, or null when none was declared or resolved.
 * @returns What it compiles to, opaque and undetailed for anything the backend could not describe.
 */
export function toLevelSurfaceRender(descriptor: Nullable<XraySurfaceDescriptor>): ILevelSurfaceRender {
  const detail: Maybe<XraySurfaceDescriptor["detail"]> = descriptor?.detail;
  const bump: Maybe<XraySurfaceDescriptor["bump"]> = descriptor?.bump;

  return {
    ...toRenderSurfaceDraw(descriptor),
    bump: bump ? { bump: bump.bump.reference, companion: bump.companion.reference } : null,
    // Dropped where it carries no tiling: the engine binds no scaler there either, and none can be invented for it.
    detail:
      detail && detail.scale !== null
        ? {
            bump: detail.bump ? { bump: detail.bump.bump.reference, companion: detail.bump.companion.reference } : null,
            reference: detail.reference,
            scale: detail.scale,
          }
        : null,
    isWallmark: isWallmarkSurface(descriptor),
    material: descriptor?.material ?? DEFAULT_MATERIAL,
    waterTextures: descriptor?.draw.kind === EXraySurfaceDraw.WATER ? toWaterTextures(descriptor.samplers) : null,
  };
}

/** A water script's samplers, by the role each plays. */
function toWaterTextures(samplers: ReadonlyArray<XraySurfaceSampler>): ILevelSurfaceWaterTextures {
  return {
    base: findSamplerTexture(samplers, BASE_ELEMENT, "s_base"),
    distortion: findSamplerTexture(samplers, DISTORTION_ELEMENT, "s_distort"),
    foam: findSamplerTexture(samplers, BASE_ELEMENT, "s_leaves"),
    normal: findSamplerTexture(samplers, BASE_ELEMENT, "s_nmap"),
  };
}

/**
 * @param surfaces - What the backend resolved, in the level's shader table order.
 * @param shaderId - The entry, as a packed surface names it.
 * @returns What it compiles to, opaque for an id the table does not reach.
 */
export function getLevelSurfaceRender(
  surfaces: ReadonlyArray<XraySurfaceDescriptor>,
  shaderId: number
): ILevelSurfaceRender {
  return toLevelSurfaceRender(surfaces[shaderId] ?? null);
}

/**
 * Where in the frame an entry is drawn, as a panel names it.
 *
 * @param render - What the entry compiles to.
 * @returns The pass, in words.
 */
export function describeLevelSurfacePass(render: ILevelSurfaceRender): string {
  switch (toRenderPass(render.draw, render.isWallmark)) {
    case ERenderPass.DEFERRED:
      return "the G-buffer, lit by the sun and the hemisphere";

    case ERenderPass.WALLMARK:
      return "the albedo, before any light reaches it";

    case ERenderPass.FORWARD:
      return render.isLit ? "over the lit frame, lit itself" : "over the lit frame, unlit";

    case ERenderPass.WATER:
      return "over the lit frame as water, reflecting the sky and distorting what is behind it";
  }
}
