import { RenderFrameRate } from "@/core/ipc/types/xrf-renderer";

/**
 * How often a viewport is allowed to redraw: a cap in frames a second, or none.
 */
export type TFrameRateLimit = "30" | "60" | "120" | "160" | "none";

/** The choices offered, coarse on purpose: this is a budget, not a tuning knob. */
export const FRAME_RATE_LIMITS: ReadonlyArray<TFrameRateLimit> = ["none", "30", "60", "120", "160"];

/** No cap: with vsync on, which it is by default, every viewport draws at the display's own refresh rate. */
export const DEFAULT_FRAME_RATE_LIMIT: TFrameRateLimit = "none";

/**
 * Reads a stored choice, falling back to the default rather than trusting what is in storage.
 *
 * @param stored - What local storage holds, which is a string or nothing at all.
 * @returns One of the offered limits.
 */
export function toFrameRateLimit(stored: unknown): TFrameRateLimit {
  return FRAME_RATE_LIMITS.find((limit: TFrameRateLimit) => limit === stored) ?? DEFAULT_FRAME_RATE_LIMIT;
}

/**
 * @param limit - The budget chosen.
 * @param isVsync - Whether a frame waits for the display's refresh.
 * @returns The rate the renderer draws every viewport at.
 */
export function toRenderFrameRate(limit: TFrameRateLimit, isVsync: boolean): RenderFrameRate {
  return { isVsync, limit: limit === "none" ? null : Number(limit) };
}
