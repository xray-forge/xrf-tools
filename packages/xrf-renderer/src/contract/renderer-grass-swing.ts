/**
 * One way the grass swings, as `[details]` in `system.ltx` states it: two winds turning at their own rates, and a wave
 * running through the level.
 */
export interface IRendererGrassSwing {
  /** How far the first and second wind lean the grass. */
  amp1: number;
  amp2: number;
  /** Seconds each wind takes to turn once around. */
  rot1: number;
  rot2: number;
  /** How fast the wave runs. */
  speed: number;
}
