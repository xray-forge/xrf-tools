import { clamp } from "@xrf/math";

import { DEFAULT_LEVEL_FLY_OPTIONS, ILevelFlyOptions } from "@/core/level/lib/camera/level-fly-options";

/**
 * How the camera sees and how it answers input, as one value the toolbar owns.
 */
export interface ILevelCameraOptions extends ILevelFlyOptions {
  /** Vertical field of view in degrees. */
  fieldOfView: number;
}

/**
 * What the camera sees and how fast it moves before anyone touches it.
 */
export const DEFAULT_LEVEL_CAMERA_OPTIONS: ILevelCameraOptions = {
  ...DEFAULT_LEVEL_FLY_OPTIONS,
  fieldOfView: 67.5,
};

/** The bounds each value is offered between, so the control and the scene agree on what is askable. */
export const LEVEL_CAMERA_LIMITS = {
  /** Past a hundred the projection distorts more than it shows; under thirty it is a telescope. */
  fieldOfView: { max: 120, min: 30, step: 1 },
  /** Times the walking speed the modifier multiplies by, which a level the size of Zaton needs. */
  boost: { max: 20, min: 1, step: 0.5 },
  /** Radians of turn per pixel of pointer movement. */
  sensitivity: { max: 0.01, min: 0.0005, step: 0.0005 },
  /** Metres a second at a walk. */
  speed: { max: 120, min: 1, step: 1 },
} as const;

/**
 * @param stored - What was stored for the camera, parsed from wherever it is kept.
 * @returns The options, each value held to its bounds and the default for any value that is not a number.
 */
export function toLevelCameraOptions(stored: unknown): ILevelCameraOptions {
  const source: Partial<Record<keyof ILevelCameraOptions, unknown>> =
    typeof stored === "object" && stored !== null ? stored : {};

  function toValue(key: keyof typeof LEVEL_CAMERA_LIMITS): number {
    const value: unknown = source[key];
    const { min, max } = LEVEL_CAMERA_LIMITS[key];

    return typeof value === "number" && Number.isFinite(value)
      ? clamp(value, min, max)
      : DEFAULT_LEVEL_CAMERA_OPTIONS[key];
  }

  return {
    boost: toValue("boost"),
    fieldOfView: toValue("fieldOfView"),
    sensitivity: toValue("sensitivity"),
    speed: toValue("speed"),
  };
}
