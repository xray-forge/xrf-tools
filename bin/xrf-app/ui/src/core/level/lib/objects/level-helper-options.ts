import { LEVEL_OBJECT_KINDS, LevelObjectKind, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { clamp } from "@xrf/math";
import { isRecord } from "@xrf/types";

import { getLevelObjectKind } from "@/core/level/lib/objects/level-object-kinds";
import { IRenderLimits } from "@/core/render/lib/features/render-limits";

/** Which helpers the viewer draws over its level objects, and how: a preference, kept over runs. */
export interface ILevelHelperOptions {
  /** Draws the helpers at all: off is the photo mode, on the inspector, each kind keeping its choice either way. */
  isShown: boolean;
  /** Each kind's helper as switched, by kind name; a kind not named draws as the kind table says by default. */
  kinds: Partial<Record<LevelObjectKind, boolean>>;
  /** How strongly a wire shows behind what the level draws in front of it, from none to whole. */
  occludedStrength: number;
  /** What every kind's draw distance is scaled by. */
  distanceScale: number;
}

/** How strongly an occluded wire shows: from none to whole. */
export const LEVEL_HELPER_OCCLUDED_LIMITS: IRenderLimits = { max: 1, min: 0, step: 0.05 };

/** How far every kind's draw distance is scaled: from a quarter to four times. */
export const LEVEL_HELPER_DISTANCE_LIMITS: IRenderLimits = { max: 4, min: 0.25, step: 0.25 };

/** The photo mode, every kind left to the kind table, an occluded wire at a quarter. */
export const DEFAULT_LEVEL_HELPER_OPTIONS: ILevelHelperOptions = {
  distanceScale: 1,
  isShown: false,
  kinds: {},
  occludedStrength: 0.25,
};

/**
 * @param options - Which helpers are drawn.
 * @param kind - A kind of level object.
 * @returns Whether its helper is switched on, the master switch aside.
 */
export function isLevelHelperOn(options: ILevelHelperOptions, kind: LevelObjectKind): boolean {
  return options.kinds[kind] ?? getLevelObjectKind(kind).isHelpedByDefault;
}

/** A stored number within its limits, or the default where none was stored. */
function toLimited(stored: unknown, limits: IRenderLimits, fallback: number): number {
  return typeof stored === "number" && Number.isFinite(stored) ? clamp(stored, limits.min, limits.max) : fallback;
}

/**
 * @param stored - The parsed stored options, `null` where none were stored.
 * @returns What was stored, a switch it does not hold or a kind the table no longer has left to its default.
 */
export function toLevelHelperOptions(stored: unknown): ILevelHelperOptions {
  if (!isRecord(stored)) {
    return DEFAULT_LEVEL_HELPER_OPTIONS;
  }

  const kinds: Record<string, unknown> = isRecord(stored.kinds) ? stored.kinds : {};

  return {
    distanceScale: toLimited(
      stored.distanceScale,
      LEVEL_HELPER_DISTANCE_LIMITS,
      DEFAULT_LEVEL_HELPER_OPTIONS.distanceScale
    ),
    isShown: typeof stored.isShown === "boolean" ? stored.isShown : DEFAULT_LEVEL_HELPER_OPTIONS.isShown,
    kinds: Object.fromEntries(
      LEVEL_OBJECT_KINDS.filter((entry: LevelObjectKindEntry) => typeof kinds[entry.kind] === "boolean").map(
        (entry: LevelObjectKindEntry) => [entry.kind, kinds[entry.kind]]
      )
    ),
    occludedStrength: toLimited(
      stored.occludedStrength,
      LEVEL_HELPER_OCCLUDED_LIMITS,
      DEFAULT_LEVEL_HELPER_OPTIONS.occludedStrength
    ),
  };
}
