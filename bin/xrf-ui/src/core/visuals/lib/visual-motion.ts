/** Frames a second an X-Ray motion samples at, which is what playback has to run at to look right. */
export const MOTION_SAMPLE_FPS: number = 30;

/**
 * Rates playback is allowed to run at.
 *
 * A floor of one frame a second because zero is what pause is for, and a ceiling of a hundred and twenty because a
 * `setInterval` cannot keep a shorter period and a viewer has nothing to learn from frames it cannot see.
 */
const MIN_FPS: number = 1;
const MAX_FPS: number = 120;

/**
 * Holds a requested playback rate to what a ticker can actually keep.
 *
 * @param fps - Rate asked for.
 * @returns The rate playback will run at.
 */
export function clampMotionFps(fps: number): number {
  return Math.max(MIN_FPS, Math.min(fps, MAX_FPS));
}
