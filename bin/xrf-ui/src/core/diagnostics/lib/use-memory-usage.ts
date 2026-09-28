import { Nullable } from "@xrf/types";

import { systemCommands } from "@/core/ipc/commands/system";
import { MemoryUsage } from "@/core/ipc/types/xrf-app";
import { Logger, useLogger } from "@/lib/logging";
import { usePolledValue } from "@/lib/react";

/** How often memory is read: slow enough to cost nothing, fast enough to watch a level stream in. */
const MEMORY_POLL_INTERVAL: number = 2_000;

/**
 * Reads what the backend and the webview's processes hold, for as long as the caller is mounted and the window shown.
 *
 * @param select - Turns each reading into what the caller keeps, called as the reading lands.
 * @returns The most recent selection, or `null` before the first reading and on a platform that cannot say.
 */
export function useMemoryUsage<T>(select: (usage: MemoryUsage) => T): Nullable<T> {
  const log: Logger = useLogger(__MODULE_NAME__);

  return usePolledValue(
    () =>
      systemCommands.getMemoryUsage().then(
        (usage: Nullable<MemoryUsage>) => (usage ? select(usage) : null),
        (error: unknown) => {
          log.error("Failed to read memory usage:", error);

          return null;
        }
      ),
    MEMORY_POLL_INTERVAL
  );
}
