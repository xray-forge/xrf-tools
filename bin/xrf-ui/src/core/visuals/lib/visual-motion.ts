import { Nullable } from "@/lib/types/general";

/** Frames a second an X-Ray motion samples at, which is what playback has to run at to look right. */
export const MOTION_SAMPLE_FPS: number = 30;

/** The playback speed a motion declares when it plays at the rate it was sampled at. */
export const MOTION_DEFAULT_SPEED: number = 1;

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

/**
 * Says what a motion's reported duration was measured from.
 *
 * A duration is the motion's frames over the rate it samples at, divided by the speed its definition declares, so a
 * motion that plays fast reports less time than its frames span. That is the case a reader takes for an error, which
 * is why the speed is named only when it is not the default one - and why neither is the playback rate the viewer
 * offers, which changes how long looking at a motion takes and nothing about the motion.
 *
 * @param frameCount - Frames the motion holds.
 * @param speed - Playback speed the motion declares, as the bake reports it.
 * @returns A sentence naming what the duration beside it came from.
 */
export function formatMotionTiming(frameCount: number, speed: Nullable<number>): string {
  const sampled: string = `${frameCount} frames at ${MOTION_SAMPLE_FPS} fps`;

  return speed === null || speed === MOTION_DEFAULT_SPEED ? sampled : `${sampled}, played at speed ${speed}`;
}
