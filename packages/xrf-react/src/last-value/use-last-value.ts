import { Nullable } from "@xrf/types";
import { useRef } from "react";

/**
 * Holds the last value given so a view can keep showing it while the next one loads.
 *
 * @param current - The value for what is selected now, absent while it loads.
 * @param isStale - Whether to keep the previous value rather than forget it, normally the loading flag.
 * @returns The current value, or the last one seen while the next is on its way.
 */
export function useLastValue<T>(current: Nullable<T>, isStale: boolean): Nullable<T> {
  const last = useRef<Nullable<T>>(null);

  if (current !== null) {
    last.current = current;
  } else if (!isStale) {
    // A selection that resolved to nothing is an answer, not a gap: holding the old value there would misreport it.
    last.current = null;
  }

  return last.current;
}
