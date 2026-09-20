import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { LevelViewportService } from "@/core/level/services";
import { RenderViewportOverlay } from "@/core/render/components/overlay";
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
  const stats: ILevelStats = viewport.stats;

  return (
    <RenderViewportOverlay data-testid={dataTestId} id={id} className={className} corner={"top-left"}>
      <div>{`${stats.framesPerSecond.toFixed(0)} fps · ${stats.frameTime.toFixed(1)} ms`}</div>
      <div>{`${stats.draws} draws · ${stats.triangles.toLocaleString()} tris`}</div>
      <div>{`${stats.sectors} sectors · ${formatBytes(stats.bytes)}`}</div>
    </RenderViewportOverlay>
  );
}
