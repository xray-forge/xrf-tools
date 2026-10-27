import { saturate } from "@xrf/math";
import { Nullable } from "@xrf/types";
import { useCallback } from "react";

import { IStoredValueCodec, MEDIA_VOLUME_STORAGE_KEY, textCodec, useStoredValue } from "@/core/storage";

const DEFAULT_VOLUME: number = 1;

const VOLUME_CODEC: IStoredValueCodec<number> = textCodec(readStoredVolume);

/**
 * Reads the stored level, falling back to full volume.
 *
 * @param stored - The stored level, or `null` when there is none.
 * @returns The stored level, or the default when there is nothing usable to read.
 */
function readStoredVolume(stored: Nullable<string>): number {
  const parsed: number = stored?.trim() ? Number(stored) : Number.NaN;

  return Number.isFinite(parsed) && parsed >= 0 && parsed <= 1 ? parsed : DEFAULT_VOLUME;
}

/**
 * Holds a level inside the range a media element accepts.
 *
 * @param value - Requested level.
 * @returns The level clamped to `[0, 1]`.
 */
function clampVolume(value: number): number {
  return Number.isFinite(value) ? saturate(value) : DEFAULT_VOLUME;
}

export interface IMediaVolume {
  /** Level in `[0, 1]`, as a media element's own `volume` expresses it. */
  value: number;
  set: (next: number) => void;
}

/**
 * Remembers how loud playback should be, across selections and across restarts.
 *
 * @returns The stored level and a setter that persists it.
 */
export function useMediaVolume(): IMediaVolume {
  const [value, setValue] = useStoredValue<number>(MEDIA_VOLUME_STORAGE_KEY, VOLUME_CODEC);

  const set = useCallback((next: number): void => setValue(clampVolume(next)), [setValue]);

  return { value, set };
}
