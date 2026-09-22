import { Nullable } from "@xrf/types";
import { useEffect, useRef, useState } from "react";

import { Logger } from "@/lib/logging";

/**
 * Re-reads a value on an interval for as long as the component is mounted.
 *
 * @param read - Reads the current value, synchronously or not.
 * @param intervalMs - How long to wait between reads.
 * @returns The most recent value, or `null` before the first read lands.
 */
export function usePolledValue<T>(read: () => T | Promise<T>, intervalMs: number): Nullable<T> {
  const [value, setValue] = useState<Nullable<T>>(null);
  const readRef = useRef<() => T | Promise<T>>(read);

  readRef.current = read;

  useEffect(() => {
    let isMounted: boolean = true;
    let isReading: boolean = false;

    async function poll(): Promise<void> {
      if (isReading) {
        return;
      }

      isReading = true;

      try {
        const next: T = await readRef.current();

        if (isMounted) {
          setValue(next);
        }
      } catch (error) {
        Logger.warn("Unexpected error during polling:", error);
      } finally {
        isReading = false;
      }
    }

    void poll();

    const timer: ReturnType<typeof setInterval> = setInterval(() => void poll(), intervalMs);

    return () => {
      isMounted = false;

      clearInterval(timer);
    };
  }, [intervalMs]);

  return value;
}
