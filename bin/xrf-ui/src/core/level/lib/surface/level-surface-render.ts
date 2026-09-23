import { ERendererPass, toRendererPass } from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import {
  IRendererSurfaceDraw,
  isWallmarkSurface,
  toRendererSurfaceDraw,
} from "@/core/render/lib/surface/renderer-surface-draw";

/**
 * The detail texture a surface modulates its base with, as the shader table's answer named it.
 */
export interface ILevelSurfaceDetail {
  /** Texture reference, engine-style. */
  reference: string;
  /** Times it repeats across the surface's base coordinate. */
  scale: number;
}

/**
 * What one shader table entry compiles to, for the renderer: its draw, whether it is a wall mark, and its detail.
 */
export interface ILevelSurfaceRender extends IRendererSurfaceDraw {
  /** Whether a scripted pass says it is a wall mark, which the renderer composites into the albedo before light. */
  isWallmark: boolean;
  /** The detail bound beside the base, or null for a surface the engine details with none. */
  detail: Nullable<ILevelSurfaceDetail>;
}

/**
 * @param descriptor - What the backend resolved for the surface, or null when none was declared or resolved.
 * @returns What it compiles to, opaque and undetailed for anything the backend could not describe.
 */
export function toLevelSurfaceRender(descriptor: Nullable<XraySurfaceDescriptor>): ILevelSurfaceRender {
  const detail: Maybe<XraySurfaceDescriptor["detail"]> = descriptor?.detail;

  return {
    ...toRendererSurfaceDraw(descriptor),
    // Dropped where it carries no tiling: the engine binds no scaler there either, and none can be invented for it.
    detail: detail && detail.scale !== null ? { reference: detail.reference, scale: detail.scale } : null,
    isWallmark: isWallmarkSurface(descriptor),
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
  switch (toRendererPass(render)) {
    case ERendererPass.DEFERRED:
      return "the G-buffer, lit by the sun and the hemisphere";

    case ERendererPass.WALLMARK:
      return "the albedo, before any light reaches it";

    case ERendererPass.FORWARD:
      return render.isLit ? "over the lit frame, lit itself" : "over the lit frame, unlit";
  }
}
