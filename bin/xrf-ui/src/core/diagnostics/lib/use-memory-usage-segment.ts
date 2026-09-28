import { Nullable } from "@xrf/types";

import { TMemoryDetailSource } from "@/core/diagnostics/lib/memory-detail-source";
import { describeMemoryUsage } from "@/core/diagnostics/lib/memory-usage-segment";
import { useMemoryUsage } from "@/core/diagnostics/lib/use-memory-usage";
import { MemoryUsage } from "@/core/ipc/types/xrf-app";
import { IEditorStatusDetail } from "@/core/shell/editor-shell/editor-status-detail";
import { IEditorStatusSegment } from "@/core/shell/editor-shell/editor-status-segment";

/**
 * Polls memory into a status bar segment, each source adding its row to the hover as every reading lands.
 *
 * @param sources - Further hover rows, in order; the latest list is read, so it need not be stable.
 * @returns The segment, or `null` before the first reading and on a platform that cannot say.
 */
export function useMemoryUsageSegment(sources: ReadonlyArray<TMemoryDetailSource>): Nullable<IEditorStatusSegment> {
  return useMemoryUsage((usage: MemoryUsage) =>
    describeMemoryUsage(
      usage,
      sources
        .map((source: TMemoryDetailSource) => source())
        .filter((detail: Nullable<IEditorStatusDetail>): detail is IEditorStatusDetail => detail !== null)
    )
  );
}
