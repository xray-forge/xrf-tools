import { useEffect, useRef, useState } from "react";

import { Nullable } from "@/lib/types/general";

/**
 * Re-reads a value on an interval for as long as the component is mounted.
 *
 * Owns the part of polling that is easy to get wrong: an answer that arrives after the component is gone is dropped
 * rather than written, and the timer is cleared with the effect that started it. The first read happens immediately,
 * so a surface is not blank for one interval before saying anything.
 *
 * The reader is read from a ref, so a closure rebuilt on every render does not restart the interval; only a changed
 * interval does. That means each poll sees whatever the reader closed over most recently, which is what a poll wants.
 *
 * A reader that rejects leaves the previous value in place and the polling continues, because one refused read of a
 * live figure is not a reason to blank a panel or to stop asking. Nothing is reported from here - only the reader knows
 * what failed and how to say so - so a reader whose failures matter catches and logs inside itself.
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

    function poll(): void {
      void Promise.resolve(readRef.current()).then(
        (it: T) => {
          if (isMounted) {
            setValue(it);
          }
        },
        () => undefined
      );
    }

    poll();

    const timer: ReturnType<typeof setInterval> = setInterval(poll, intervalMs);

    return () => {
      isMounted = false;

      clearInterval(timer);
    };
  }, [intervalMs]);

  return value;
}
