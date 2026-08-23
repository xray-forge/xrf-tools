import { useCallback, useState } from "react";

import { getLocalStorageValue, setLocalStorageValue } from "@/lib/local-storage";
import { Nullable } from "@/lib/types/general";

/** One level for every media surface: setting playback quiet is a statement about the application, not about a file. */
const VOLUME_STORAGE_KEY: string = "xrf.media.volume";

const DEFAULT_VOLUME: number = 1;

export interface IMediaVolume {
  /** Level in `[0, 1]`, as a media element's own `volume` expresses it. */
  value: number;
  set: (next: number) => void;
}

/**
 * Remembers how loud playback should be, across selections and across restarts.
 *
 * Persisted rather than held in component state because the preview panel unmounts between selections - it swaps itself
 * for a progress indicator while the next file loads - so anything the player owned would reset on every click.
 *
 * @returns The stored level and a setter that persists it.
 */
export function useMediaVolume(): IMediaVolume {
  const [value, setValue] = useState<number>(readStoredVolume);

  const set = useCallback((next: number): void => {
    const clamped: number = clampVolume(next);

    setValue(clamped);
    setLocalStorageValue(VOLUME_STORAGE_KEY, String(clamped));
  }, []);

  return { value, set };
}

/**
 * Reads the stored level, falling back to full volume.
 *
 * Anything unparseable or out of range is treated as absent rather than corrected into silence: a zero recovered from a
 * bad value would look like broken playback and send the user hunting for a mute button.
 *
 * @returns The stored level, or the default when there is nothing usable to read.
 */
function readStoredVolume(): number {
  const raw: Nullable<string> = getLocalStorageValue(VOLUME_STORAGE_KEY);
  const parsed: number = raw === null ? Number.NaN : Number.parseFloat(raw);

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
