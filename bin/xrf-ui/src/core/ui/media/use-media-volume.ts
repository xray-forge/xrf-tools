import { Nullable } from "@xrf/types";
import { useCallback, useState } from "react";

import { MEDIA_VOLUME_STORAGE_KEY } from "@/core/storage";
import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";

const DEFAULT_VOLUME: number = 1;

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
  const [value, setValue] = useState<number>(readStoredVolume);

  const set = useCallback((next: number): void => {
    const clamped: number = clampVolume(next);

    setValue(clamped);
    setLocalStorageValue(MEDIA_VOLUME_STORAGE_KEY, String(clamped));
  }, []);

  return { value, set };
}

/**
 * Reads the stored level, falling back to full volume.
 *
 * @returns The stored level, or the default when there is nothing usable to read.
 */
function readStoredVolume(): number {
  const raw: Nullable<string> = getLocalStorageValue(MEDIA_VOLUME_STORAGE_KEY);
  const parsed: number = raw?.trim() ? Number(raw) : Number.NaN;

  return Number.isFinite(parsed) && parsed >= 0 && parsed <= 1 ? parsed : DEFAULT_VOLUME;
}

/**
 * Holds a level inside the range a media element accepts.
 *
 * @param value - Requested level.
 * @returns The level clamped to `[0, 1]`.
 */
function clampVolume(value: number): number {
  return Number.isFinite(value) ? Math.min(1, Math.max(0, value)) : DEFAULT_VOLUME;
}
