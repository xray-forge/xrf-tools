/**
 * How often a viewport is allowed to redraw.
 */
export type TFrameRateLimit = "30" | "60" | "120" | "160" | "unlimited";

/** The choices offered, coarse on purpose: this is a budget, not a tuning knob. */
export const FRAME_RATE_LIMITS: ReadonlyArray<TFrameRateLimit> = ["30", "60", "120", "160", "unlimited"];

/** Enough for anything a viewer does, and a third of what an unthrottled loop costs on a fast display. */
export const DEFAULT_FRAME_RATE_LIMIT: TFrameRateLimit = "60";

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
 * Milliseconds between two frames at a limit.
 *
 * @param limit - The budget chosen.
 * @returns The interval, or zero for a viewport allowed every frame.
 */
export function toFrameInterval(limit: TFrameRateLimit): number {
  return limit === "unlimited" ? 0 : 1000 / Number(limit);
}
