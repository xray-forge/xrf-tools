import { Maybe, Nullable } from "@xrf/types";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { DEFAULT_LEVEL_WEATHER_CONTROL, ILevelWeatherControl } from "@/core/level/lib/weather/level-weather-control";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LEVEL_WEATHER_DAY } from "@/core/level/lib/weather/level-weather-time";
import { LEVEL_WEATHER_STORAGE_KEY } from "@/core/storage";
import { parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/lib/local-storage";

/** Levels remembered at most, the least recently played let go first. */
const REMEMBERED_LEVELS: number = 32;

/** The engine's own bounds on its time factor. */
export const LEVEL_WEATHER_FACTOR_LIMITS = { max: 1000, min: 1 } as const;

/**
 * How a level's weather was last played, which it plays again when it opens.
 */
export interface ILevelWeatherMemory {
  source: ELevelWeatherSource;
  /** The cycle's name. */
  cycle: string;
  /** Seconds since midnight. */
  time: number;
  control: ILevelWeatherControl;
}

/**
 * @param level - An open level.
 * @returns What its memory is kept under: the level and the roots it was opened in, so two games' levels of one name
 *   are remembered apart.
 */
export function toLevelWeatherMemoryKey(level: SelectedLevelDescription): string {
  const source: string = level.source.kind === "asset" ? level.source.logicalPath : level.source.path;

  return [...level.roots.roots.map((root) => root.path), source].join("|").toLowerCase();
}

/**
 * @param key - The level's memory key.
 * @returns How it was last played, or null for a level never played or a memory that no longer reads.
 */
export function readLevelWeatherMemory(key: string): Nullable<ILevelWeatherMemory> {
  return toLevelWeatherMemory(readMemories()[key]);
}

/**
 * @param key - The level's memory key.
 * @param memory - How it plays now.
 */
export function writeLevelWeatherMemory(key: string, memory: ILevelWeatherMemory): void {
  const memories: Record<string, unknown> = readMemories();

  delete memories[key];

  // Insertion order is recency: the newest last, the oldest first to go.
  const entry: [string, unknown] = [key, memory];
  const kept: Array<[string, unknown]> = [...Object.entries(memories), entry].slice(-REMEMBERED_LEVELS);

  setLocalStorageValueSafe(LEVEL_WEATHER_STORAGE_KEY, JSON.stringify(Object.fromEntries(kept)));
}

/**
 * @param stored - A value read back from storage.
 * @returns The memory it holds, every field checked, or null where it does not read as one.
 */
export function toLevelWeatherMemory(stored: unknown): Nullable<ILevelWeatherMemory> {
  if (!isRecord(stored) || !isRecord(stored.control)) {
    return null;
  }

  const { source, cycle, time, control } = stored;
  const factor: Maybe<unknown> = control.factor;

  if (
    !Object.values(ELevelWeatherSource).includes(source as ELevelWeatherSource) ||
    typeof cycle !== "string" ||
    typeof time !== "number" ||
    !Number.isFinite(time)
  ) {
    return null;
  }

  return {
    control: {
      factor:
        typeof factor === "number" && Number.isFinite(factor)
          ? Math.min(Math.max(factor, LEVEL_WEATHER_FACTOR_LIMITS.min), LEVEL_WEATHER_FACTOR_LIMITS.max)
          : DEFAULT_LEVEL_WEATHER_CONTROL.factor,
      isDynamicSun: control.isDynamicSun === true,
      isPaused: control.isPaused !== false,
    },
    cycle,
    source: source as ELevelWeatherSource,
    time: Math.min(Math.max(time, 0), LEVEL_WEATHER_DAY - 1),
  };
}

function readMemories(): Record<string, unknown> {
  const stored: unknown = parseLocalStorageValueSafe(LEVEL_WEATHER_STORAGE_KEY);

  return isRecord(stored) ? { ...stored } : {};
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
