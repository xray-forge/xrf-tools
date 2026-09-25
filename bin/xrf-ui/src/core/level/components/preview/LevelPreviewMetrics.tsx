import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { LevelViewportService, LevelViewService } from "@/core/level/services";
import { RenderFrameReadout } from "@/core/render/components/overlay";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatBytes } from "@/lib/memory/format";

/**
 * What the frame just drawn cost, laid over the viewport that drew it.
 */
export function LevelPreviewMetrics({
  "data-testid": dataTestId = "level-preview-metrics",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const viewport: LevelViewportService = useInjection(LevelViewportService);
  const view: LevelViewService = useInjection(LevelViewService);
  const stats: ILevelStats = viewport.stats;

  return (
    <RenderFrameReadout
      data-testid={dataTestId}
      id={id}
      className={className}
      cost={stats}
      timings={viewport.timings}
      isAdvanced={view.options.isAdvancedStatsVisible}
    >
      <div>{`${stats.sectors} sectors · ${formatBytes(stats.bytes)}`}</div>
    </RenderFrameReadout>
  );
}
