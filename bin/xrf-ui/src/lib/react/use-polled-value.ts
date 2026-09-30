import { Nullable } from "@xrf/types";
import { useEffect, useRef, useState } from "react";

import { Logger } from "@/lib/logging";

/**
 * Re-reads a value on an interval for as long as the component is mounted and the document is visible.
 *
 * A hidden document (a minimized window) skips its ticks and reads once as soon as it is shown again, since nobody is
 * looking at the value in between.
 *
 * @param read - Reads the current value, synchronously or not.
 * @param intervalMs - How long to wait between reads.
 * @returns The most recent value, or `null` before the first read lands.
 */
export function usePolledValue<T>(read: () => T | Promise<T>, intervalMs: number): Nullable<T> {
  const [value, setValue] = useState<Nullable<T>>(null);
  const readRef = useRef<() => T | Promise<T>>(read);
  const valueRef = useRef<Nullable<T>>(null);

  readRef.current = read;

  useEffect(() => {
    let isMounted: boolean = true;
    let isReading: boolean = false;

    async function poll(): Promise<void> {
      if (isReading || document.visibilityState === "hidden") {
        return;
      }

      isReading = true;

      try {
        const next: T = await readRef.current();

        // A reading equal to the last one schedules nothing, where React would only sometimes bail out of it.
        if (isMounted && !Object.is(next, valueRef.current)) {
          valueRef.current = next;
          setValue(next);
        }
      } catch (error) {
        Logger.warn("Unexpected error during polling:", error);
      } finally {
        isReading = false;
      }
    }

    function onVisibilityChange(): void {
      void poll();
    }

    void poll();

    const timer: ReturnType<typeof setInterval> = setInterval(() => void poll(), intervalMs);

    document.addEventListener("visibilitychange", onVisibilityChange);

    return () => {
      isMounted = false;

      clearInterval(timer);
      document.removeEventListener("visibilitychange", onVisibilityChange);
    };
  }, [intervalMs]);

  return value;
}
