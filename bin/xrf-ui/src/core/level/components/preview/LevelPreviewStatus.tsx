import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { readPageScriptHeapDetail, TMemoryDetailSource, useMemoryUsageSegment } from "@/core/diagnostics/lib";
import { toLevelRendererMemoryDetail } from "@/core/level/lib/stats/level-renderer-memory";
import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { LevelViewportService } from "@/core/level/services";
import { IEditorStatusSegment, TEditorStatusSegment, useEditorStatus } from "@/core/shell/editor-shell";
import { formatBytes } from "@/lib/memory/format";

interface ILevelPreviewStatusProps {
  /** What the viewer is doing, when that is worth saying instead of what it is drawing. */
  activity?: Nullable<string>;
}

/**
 * Publishes what the level viewer is drawing, and what the application holds in memory, to the application status bar.
 */
export function LevelPreviewStatus({ activity = null }: ILevelPreviewStatusProps): ReactElement {
  const viewport: LevelViewportService = useInjection(LevelViewportService);
  const stats: ILevelStats = viewport.stats;
  // What the memory hover adds after the processes: the page's heap, then the renderer's CPU copies.
  const memorySources: ReadonlyArray<TMemoryDetailSource> = useMemo(
    () => [readPageScriptHeapDetail, () => toLevelRendererMemoryDetail(viewport.stats.rendererMemory)],
    [viewport]
  );
  const memorySegment: Nullable<IEditorStatusSegment> = useMemoryUsageSegment(memorySources);

  const segments: Array<TEditorStatusSegment> = useMemo(
    () => [
      // Concatenated rather than swapped for: a status line that replaces its whole content while streaming moves
      // every word in it, and the words a person is reading are the ones that were already there.
      ...(activity ? [activity] : []),
      `${stats.sectors} sectors`,
      formatBytes(stats.bytes),
      ...(memorySegment ? [memorySegment] : []),
    ],
    [activity, stats, memorySegment]
  );

  useEditorStatus(segments);

  // JSX is needed to wrap with `observer` automatically, `null` kills reactivity.
  return <></>;
}
