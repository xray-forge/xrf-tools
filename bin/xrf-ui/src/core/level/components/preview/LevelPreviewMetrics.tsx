import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LevelRenderService, LevelViewportService } from "@/core/level/services";
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
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const viewportService: LevelViewportService = useInjection(LevelViewportService);
  const sectors: number = viewportService.load?.sectors ?? 0;
  const bytes: number = viewportService.load?.bytes ?? 0;

  return (
    <RenderFrameReadout data-testid={dataTestId} id={id} className={className} report={renderService.frame}>
      <div>{`${sectors} sectors · ${formatBytes(bytes)}`}</div>
    </RenderFrameReadout>
  );
}
