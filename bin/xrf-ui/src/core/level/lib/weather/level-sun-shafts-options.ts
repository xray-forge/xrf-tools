import { clamp } from "@xrf/math";

import { RenderSunShafts } from "@/core/ipc/types/xrf-renderer";

/** How a level's sun shafts are drawn: how finely they step, and the floor under their density. */
export interface ILevelSunShaftsOptions extends RenderSunShafts {
  /** Steps along a ray, as `RenderSunShafts::STEPS_RANGE` holds them. */
  steps: number;
  /** `r2_sunshafts_min`, from zero to a half; zero draws the keyframes' density as it is. */
  minimum: number;
}

/** Monolith's high quality, 30 steps, and no floor. */
export const DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS: ILevelSunShaftsOptions = {
  minimum: 0,
  steps: 30,
};

/** The bounds `r2_sunshafts_min` is offered between, as Monolith's console holds it. */
export const LEVEL_SUN_SHAFTS_MINIMUM_LIMITS = { max: 0.5, min: 0, step: 0.01 } as const;

/**
 * The bounds the steps are offered between, the renderer's own (`RenderSunShafts::STEPS_RANGE`), and the range xrf
 * recommends: the engines' own 15 to 40.
 */
export const LEVEL_SUN_SHAFTS_STEPS_LIMITS = { max: 64, min: 8, recommended: [15, 40], step: 1 } as const;

/**
 * @param stored - What was stored for the sunshafts, parsed from wherever it is kept.
 * @returns The options, the minimum held to its bounds and the default for any value that does not read.
 */
export function toLevelSunShaftsOptions(stored: unknown): ILevelSunShaftsOptions {
  const source: Partial<Record<keyof ILevelSunShaftsOptions, unknown>> =
    typeof stored === "object" && stored !== null ? stored : {};
  const { minimum, steps } = source;
  const { min, max } = LEVEL_SUN_SHAFTS_MINIMUM_LIMITS;

  return {
    minimum:
      typeof minimum === "number" && Number.isFinite(minimum)
        ? clamp(minimum, min, max)
        : DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS.minimum,
    steps:
      typeof steps === "number" && Number.isFinite(steps)
        ? clamp(Math.round(steps), LEVEL_SUN_SHAFTS_STEPS_LIMITS.min, LEVEL_SUN_SHAFTS_STEPS_LIMITS.max)
        : DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS.steps,
  };
}
