import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { TMemoryDetailSource } from "@/core/diagnostics/lib";
import { RenderLoadReport } from "@/core/ipc/types/xrf-renderer";
import { LevelRenderService, LevelViewportService } from "@/core/level/services";
import { toNativeMemoryDetails } from "@/core/render/lib/native/native-memory-details";
import { TEditorStatusSegment, useEditorStatus, usePublishedMemoryDetails } from "@/core/shell/editor-shell";
import { formatBytes } from "@/lib/memory/format";

interface ILevelPreviewStatusProps {
  /** What the viewer is doing, when that is worth saying instead of what it is drawing. */
  activity?: Nullable<string>;
}

/**
 * Publishes what the level viewer is drawing to the application status bar, and what its renderer holds on the GPU to
 * the memory hover there.
 */
export function LevelPreviewStatus({ activity = null }: ILevelPreviewStatusProps): ReactElement {
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const load: Nullable<RenderLoadReport> = viewportService.load;
  // What the status bar's memory hover adds after the processes and the page's heap.
  const memoryDetails: ReadonlyArray<TMemoryDetailSource> = useMemo(
    () => toNativeMemoryDetails(() => renderService.frame.memory),
    [renderService]
  );

  const segments: Array<TEditorStatusSegment> = useMemo(
    () => [
      // Concatenated rather than swapped for: a status line that replaces its whole content while streaming moves
      // every word in it, and the words a person is reading are the ones that were already there.
      ...(activity ? [activity] : []),
      `${load?.sectors ?? 0} sectors`,
      formatBytes(load?.bytes ?? 0),
    ],
    [activity, load]
  );

  usePublishedMemoryDetails(memoryDetails);

  useEditorStatus(segments);

  // JSX is needed to wrap with `observer` automatically, `null` kills reactivity.
  return <></>;
}
