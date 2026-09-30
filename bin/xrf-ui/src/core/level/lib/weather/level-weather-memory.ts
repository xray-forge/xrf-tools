import { clamp } from "@xrf/math";
import { WEATHER_DAY_LENGTH } from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app";
import { ILevelManualWeather, readLevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  DEFAULT_LEVEL_WEATHER_CONTROL,
  ILevelWeatherControl,
  toLevelWeatherFactor,
} from "@/core/level/lib/weather/level-weather-control";
import { ILevelWeatherSeed } from "@/core/level/lib/weather/level-weather-seed";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LEVEL_WEATHER_STORAGE_KEY } from "@/core/storage";
import { parseLocalStorageValueSafe, setLocalStorageValueSafe } from "@/lib/local-storage";

/** Levels remembered at most, the least recently played let go first. */
const REMEMBERED_LEVELS: number = 32;

/** Characters the whole memory is kept under, the least recently played let go first past it. */
const REMEMBERED_LENGTH: number = 64 * 1024;

/** The shape a memory is written in: one of any other is let go rather than read, and nothing is migrated. */
export const LEVEL_WEATHER_MEMORY_VERSION: number = 2;

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
  /** The keyframe set by hand, or null for none set yet. */
  manual: Nullable<ILevelManualWeather>;
  /** What it was seeded from, or null for one seeded from nothing the level played. */
  seed: Nullable<ILevelWeatherSeed>;
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
  const entry: [string, unknown] = [key, { ...memory, version: LEVEL_WEATHER_MEMORY_VERSION }];
  const kept: Array<[string, unknown]> = [...Object.entries(memories), entry].slice(-REMEMBERED_LEVELS);

  let written: string = JSON.stringify(Object.fromEntries(kept));

  while (written.length > REMEMBERED_LENGTH && kept.length > 1) {
    kept.shift();
    written = JSON.stringify(Object.fromEntries(kept));
  }

  setLocalStorageValueSafe(LEVEL_WEATHER_STORAGE_KEY, written);
}

/**
 * @param stored - A value read back from storage.
 * @returns The memory it holds, every field checked, or null where it does not read as one.
 */
export function toLevelWeatherMemory(stored: unknown): Nullable<ILevelWeatherMemory> {
  if (!isRecord(stored) || !isRecord(stored.control) || stored.version !== LEVEL_WEATHER_MEMORY_VERSION) {
    return null;
  }

  const { source, cycle, time, control } = stored;
  const factor: Maybe<unknown> = control.factor;
  const manual: Nullable<ILevelManualWeather> = stored.manual === null ? null : readLevelManualWeather(stored.manual);
  const seed: Nullable<ILevelWeatherSeed> = toSeed(stored.seed);

  if (
    !Object.values(ELevelWeatherSource).includes(source as ELevelWeatherSource) ||
    typeof cycle !== "string" ||
    typeof time !== "number" ||
    !Number.isFinite(time) ||
    (stored.manual !== null && !manual)
  ) {
    return null;
  }

  return {
    control: {
      factor:
        typeof factor === "number" && Number.isFinite(factor)
          ? toLevelWeatherFactor(factor)
          : DEFAULT_LEVEL_WEATHER_CONTROL.factor,
      isDynamicSun: control.isDynamicSun === true,
      isPaused: control.isPaused !== false,
    },
    cycle,
    manual,
    seed,
    source: source as ELevelWeatherSource,
    time: clamp(time, 0, WEATHER_DAY_LENGTH - 1),
  };
}

function toSeed(stored: unknown): Nullable<ILevelWeatherSeed> {
  return isRecord(stored) && typeof stored.cycle === "string" && typeof stored.time === "number"
    ? { cycle: stored.cycle, time: stored.time }
    : null;
}

function readMemories(): Record<string, unknown> {
  const stored: unknown = parseLocalStorageValueSafe(LEVEL_WEATHER_STORAGE_KEY);

  return isRecord(stored) ? { ...stored } : {};
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
