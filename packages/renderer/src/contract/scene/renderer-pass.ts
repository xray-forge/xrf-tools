import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererSurface } from "#/contract/scene/renderer-surface";

/**
 * Which pass of the frame draws a surface, as the engine orders them.
 */
export enum ERendererPass {
  /** Into the G-buffer, lit by the deferred passes. */
  DEFERRED = "deferred",
  /** Into the G-buffer's albedo, before any light: the engine's wall mark phase. */
  WALLMARK = "wallmark",
  /** Composited over the tonemapped frame. */
  FORWARD = "forward",
  /** Composited over the lit frame before the forward surfaces, the distortion it causes written beside it. */
  WATER = "water",
}

/**
 * @param surface - What a consumer puts, or as much of it as decides this.
 * @returns The pass its draw puts it in.
 */
export function toRendererPass(surface: Pick<IRendererSurface, "draw" | "isWallmark">): ERendererPass {
  if (surface.draw === ERendererDraw.OPAQUE || surface.draw === ERendererDraw.CUT_OUT) {
    return ERendererPass.DEFERRED;
  }

  if (surface.draw === ERendererDraw.WATER) {
    return ERendererPass.WATER;
  }

  return surface.isWallmark ? ERendererPass.WALLMARK : ERendererPass.FORWARD;
}
