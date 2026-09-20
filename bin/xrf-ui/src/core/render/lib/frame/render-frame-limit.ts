import { Nullable } from "@/lib/types/general";

/**
 * How often a viewport is allowed to redraw.
 */
export type TFrameRateLimit = "30" | "60" | "120" | "unlimited";

/** The choices offered, coarse on purpose: this is a budget, not a tuning knob. */
export const FRAME_RATE_LIMITS: ReadonlyArray<TFrameRateLimit> = ["30", "60", "120", "unlimited"];

/** Enough for anything a viewer does, and a third of what an unthrottled loop costs on a fast display. */
export const DEFAULT_FRAME_RATE_LIMIT: TFrameRateLimit = "60";

/** How much of the interval has to have passed, leaving room for a wake that lands just short of it. */
const FRAME_ALLOWANCE: number = 0.9;

/**
 * Reads a stored choice, falling back to the default rather than trusting what is in storage.
 *
 * @param stored - What local storage holds, which is a string or nothing at all.
 * @returns One of the offered limits.
 */
export function toFrameRateLimit(stored: unknown): TFrameRateLimit {
  return FRAME_RATE_LIMITS.find((limit) => limit === stored) ?? DEFAULT_FRAME_RATE_LIMIT;
}

/**
 * Milliseconds a viewport has to wait before drawing again.
 *
 * @param limit - The budget chosen.
 * @returns The wait, or zero for a viewport allowed every frame.
 */
export function toFrameInterval(limit: TFrameRateLimit): number {
  return limit === "unlimited" ? 0 : (1000 / Number(limit)) * FRAME_ALLOWANCE;
}

/**
 * Whether a frame due now is worth drawing under a limit.
 *
 * @param now - The animation frame's own timestamp, in milliseconds.
 * @param drawnAt - When the last frame was drawn, or null before any has been.
 * @param limit - The budget chosen.
 * @returns Whether to draw this one.
 */
export function shouldDrawFrame(now: number, drawnAt: Nullable<number>, limit: TFrameRateLimit): boolean {
  return drawnAt === null || now - drawnAt >= toFrameInterval(limit);
}
