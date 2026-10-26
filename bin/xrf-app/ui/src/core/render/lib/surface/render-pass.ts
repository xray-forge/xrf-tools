import { ERenderDraw } from "@/core/render/lib/surface/render-draw";

/**
 * Which pass of the frame draws a surface, as the engine orders them.
 */
export enum ERenderPass {
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
 * @param draw - How a surface is drawn.
 * @param isWallmark - Whether it is a mark laid on a wall, which is composited into the albedo before light.
 * @returns The pass its draw puts it in.
 */
export function toRenderPass(draw: ERenderDraw, isWallmark: boolean = false): ERenderPass {
  if (draw === ERenderDraw.OPAQUE || draw === ERenderDraw.CUT_OUT) {
    return ERenderPass.DEFERRED;
  }

  if (draw === ERenderDraw.WATER) {
    return ERenderPass.WATER;
  }

  return isWallmark ? ERenderPass.WALLMARK : ERenderPass.FORWARD;
}
