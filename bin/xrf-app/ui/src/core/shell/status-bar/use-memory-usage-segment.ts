import { MemoryUsage } from "@xrf/ipc/types/xrf-app-system";
import { Nullable } from "@xrf/types";

import { useMemoryUsage } from "@/core/diagnostics/lib/use-memory-usage";
import { TMemoryDetailSource } from "@/core/shell/status-bar/memory-detail-source";
import { describeMemoryUsage } from "@/core/shell/status-bar/memory-usage-segment";
import { IShellStatusDetail } from "@/core/shell/status-bar/shell-status-detail";
import { IShellStatusSegment } from "@/core/shell/status-bar/shell-status-segment";

/**
 * Polls memory into a status bar segment, each source adding its row to the hover as every reading lands.
 *
 * @param sources - Further hover rows, in order; the latest list is read, so it need not be stable.
 * @returns The segment, or `null` before the first reading and on a platform that cannot say.
 */
export function useMemoryUsageSegment(sources: ReadonlyArray<TMemoryDetailSource>): Nullable<IShellStatusSegment> {
  return useMemoryUsage((usage: MemoryUsage) =>
    describeMemoryUsage(
      usage,
      sources
        .map((source: TMemoryDetailSource) => source())
        .filter((detail: Nullable<IShellStatusDetail>): detail is IShellStatusDetail => detail !== null)
    )
  );
}
