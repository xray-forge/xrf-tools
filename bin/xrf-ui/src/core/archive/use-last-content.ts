import { useRef } from "react";

import { Nullable } from "@/lib/types/general";

/**
 * Holds the last content of one kind so a panel can keep showing it while the next one loads.
 *
 * @param current - Content for the current selection, absent while it loads.
 * @param isStale - Whether to keep the previous value rather than forget it, normally the loading flag.
 * @returns The current content, or the last one seen while the next is on its way.
 */
export function useLastContent<T>(current: Nullable<T>, isStale: boolean): Nullable<T> {
  const last = useRef<Nullable<T>>(null);

  if (current) {
    last.current = current;
  } else if (!isStale) {
    // A selection that resolved to nothing is an answer, not a gap: holding the old file there would misreport it.
    last.current = null;
  }

  return last.current;
}
