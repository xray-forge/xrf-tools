import { useInjection } from "@wirestate/react";
import { Nullable } from "@xrf/types";
import { ReactElement, useMemo } from "react";

import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { LevelViewportService } from "@/core/level/services";
import { useEditorStatus } from "@/core/shell/editor-shell";
import { formatBytes } from "@/lib/memory/format";

interface ILevelPreviewStatusProps {
  /** What the viewer is doing, when that is worth saying instead of what it is drawing. */
  activity?: Nullable<string>;
}

/**
 * Publishes what the level viewer is drawing to the application status bar.
 */
export function LevelPreviewStatus({ activity = null }: ILevelPreviewStatusProps): ReactElement {
  const viewport: LevelViewportService = useInjection(LevelViewportService);
  const stats: ILevelStats = viewport.stats;

  const segments: Array<string> = useMemo(
    () => [
      // Concatenated rather than swapped for: a status line that replaces its whole content while streaming moves
      // every word in it, and the words a person is reading are the ones that were already there.
      ...(activity ? [activity] : []),
      `${stats.sectors} sectors`,
      formatBytes(stats.bytes),
    ],
    [activity, stats]
  );

  useEditorStatus(segments);

  // JSX is needed to wrap with `observer` automatically, `null` kills reactivity.
  return <></>;
}
