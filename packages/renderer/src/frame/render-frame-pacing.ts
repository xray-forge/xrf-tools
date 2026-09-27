import { DEFAULT_FRAME_RATE_LIMIT, TFrameRateLimit } from "#/frame/render-frame-limit";

/** Frames the GPU may still be working on as the next starts, with low latency on: one drawn, one queued behind it. */
export const LOW_LATENCY_FRAMES_IN_FLIGHT: number = 2;

/**
 * How frames are paced: how often one may be drawn, and how far they may run ahead of the GPU.
 */
export interface IRenderFramePacing {
  /** How often a frame may be drawn. */
  rateLimit: TFrameRateLimit;
  /**
   * Whether a frame waits until the GPU is at most one frame behind: a move shows a frame sooner when the GPU is the
   * bottleneck, for a tenth or so fewer frames. Off, the browser queues as many as it likes, about three.
   */
  isLowLatency: boolean;
}

export const DEFAULT_RENDER_FRAME_PACING: IRenderFramePacing = {
  isLowLatency: true,
  rateLimit: DEFAULT_FRAME_RATE_LIMIT,
};

/**
 * @param pacing - How frames are paced.
 * @returns Frames the GPU may still be working on as the next starts; unbounded leaves it to the browser.
 */
export function toFramesInFlight(pacing: IRenderFramePacing): number {
  return pacing.isLowLatency ? LOW_LATENCY_FRAMES_IN_FLIGHT : Infinity;
}
