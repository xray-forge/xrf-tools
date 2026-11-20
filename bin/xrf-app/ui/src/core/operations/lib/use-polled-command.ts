import { Nullable } from "@xrf/types";
import { useEffect } from "react";

import { PolledCommand } from "@/core/operations/lib/polled-command";

/**
 * Watches a polled command for as long as the caller is mounted, sharing the poll with every other watcher.
 *
 * @param polled - The command to watch.
 * @returns Its most recent reading, or null before the first lands; the caller observes it.
 */
export function usePolledCommand<T>(polled: PolledCommand<T>): Nullable<T> {
  useEffect(() => polled.watch(), [polled]);

  return polled.value;
}
