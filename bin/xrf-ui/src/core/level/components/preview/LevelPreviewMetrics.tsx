import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import { LevelRenderService } from "@/core/level/services";
import { RenderFrameReadout } from "@/core/render/components/overlay";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What the frame just drawn cost, laid over the viewport that drew it.
 */
export function LevelPreviewMetrics({
  "data-testid": dataTestId = "level-preview-metrics",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const renderService: LevelRenderService = useInjection(LevelRenderService);

  return <RenderFrameReadout data-testid={dataTestId} id={id} className={className} report={renderService.frame} />;
}
