import { useInjection } from "@wirestate/react";
import { MemoryUsage } from "@xrf/ipc/types/xrf-app-system";
import { Nullable } from "@xrf/types";

import { RuntimeUsageService } from "@/core/diagnostics/services/runtime-usage.service";
import { usePolledCommand } from "@/core/operations/lib/use-polled-command";

/**
 * Reads what the backend and the webview's processes hold while the caller is mounted and the window shown, sharing
 * one poll with every other reader.
 *
 * @returns The most recent reading, or `null` before the first and on a platform that cannot say.
 */
export function useMemoryUsage(): Nullable<MemoryUsage> {
  const runtimeUsageService: RuntimeUsageService = useInjection(RuntimeUsageService);

  return usePolledCommand(runtimeUsageService.memory);
}
