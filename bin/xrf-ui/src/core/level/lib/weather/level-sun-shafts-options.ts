import { clamp } from "@xrf/math";

import { ERenderSunShaftsQuality, RenderSunShafts, RenderSunShaftsQuality } from "@/core/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** How a level's sun shafts are drawn: how finely they step, and the floor under their density. */
export interface ILevelSunShaftsOptions extends RenderSunShafts {
  /** `r2_sunshafts_min`, from zero to a half; zero draws the keyframes' density as it is. */
  minimum: number;
  /** Steps along a ray in place of the quality's, or null to step as the quality does. */
  steps: number | null;
}

/** The engines' highest quality, stepped as it steps, and no floor, as both draw them out of the box. */
export const DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS: ILevelSunShaftsOptions = {
  minimum: 0,
  quality: ERenderSunShaftsQuality.HIGH,
  steps: null,
};

/** The bounds `r2_sunshafts_min` is offered between, as Monolith's console holds it. */
export const LEVEL_SUN_SHAFTS_MINIMUM_LIMITS = { max: 0.5, min: 0, step: 0.01 } as const;

/** The steps a ray is offered, zero standing for the quality's own. */
export const LEVEL_SUN_SHAFTS_STEPS_LIMITS = { max: 64, min: 0, step: 1 } as const;

/**
 * @param steps - Steps along a ray as the slider holds them, zero for the quality's.
 * @returns How the slider reads them.
 */
export function describeLevelSunShaftsSteps(steps: number): string {
  return steps > 0 ? String(steps) : "Quality's";
}

const LEVEL_SUN_SHAFTS_QUALITY_NAMES: Readonly<Record<RenderSunShaftsQuality, string>> = {
  [ERenderSunShaftsQuality.LOW]: "Low",
  [ERenderSunShaftsQuality.MEDIUM]: "Medium",
  [ERenderSunShaftsQuality.HIGH]: "High",
};

/** The sunshaft qualities, in the order they are offered. */
export const LEVEL_SUN_SHAFTS_QUALITY_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderSunShaftsQuality>> =
  Object.values(ERenderSunShaftsQuality).map((value: RenderSunShaftsQuality) => ({
    label: LEVEL_SUN_SHAFTS_QUALITY_NAMES[value],
    value,
  }));

/**
 * @param stored - What was stored for the sunshafts, parsed from wherever it is kept.
 * @returns The options, the minimum held to its bounds and the default for any value that does not read.
 */
export function toLevelSunShaftsOptions(stored: unknown): ILevelSunShaftsOptions {
  const source: Partial<Record<keyof ILevelSunShaftsOptions, unknown>> =
    typeof stored === "object" && stored !== null ? stored : {};
  const { minimum, quality, steps } = source;
  const { min, max } = LEVEL_SUN_SHAFTS_MINIMUM_LIMITS;

  return {
    minimum:
      typeof minimum === "number" && Number.isFinite(minimum)
        ? clamp(minimum, min, max)
        : DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS.minimum,
    quality: Object.values<unknown>(ERenderSunShaftsQuality).includes(quality)
      ? (quality as RenderSunShaftsQuality)
      : DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS.quality,
    steps:
      typeof steps === "number" && Number.isInteger(steps) && steps > 0 && steps <= LEVEL_SUN_SHAFTS_STEPS_LIMITS.max
        ? steps
        : DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS.steps,
  };
}
