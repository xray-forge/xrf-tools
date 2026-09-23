import { useInjection } from "@wirestate/react";
import { DOLLY_STEP } from "@xrf/renderer";
import { ReactElement, useCallback } from "react";

import { RenderFrameReadout } from "@/core/render/components/overlay";
import { RenderSurface } from "@/core/render/components/RenderSurface";
import { ViewportControls } from "@/core/ui/media/ViewportControls";
import { IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { VisualRenderService } from "@/core/visuals/services/visual-render.service";

/**
 * Where the open visual is drawn.
 */
export function VisualPreviewViewport(): ReactElement {
  const renderService: VisualRenderService = useInjection(VisualRenderService);
  const source: IVisualRenderSource = useInjection(VISUAL_RENDER_SOURCE);

  const onZoomIn = useCallback((): void => renderService.dolly(1 / DOLLY_STEP), [renderService]);

  const onZoomOut = useCallback((): void => renderService.dolly(DOLLY_STEP), [renderService]);

  const onReset = useCallback((): void => renderService.resetCamera(), [renderService]);

  return (
    <div className={"relative size-full"}>
      <div className={"size-full overflow-hidden"}>
        <RenderSurface host={renderService} />
      </div>

      {source.model ? (
        <>
          <RenderFrameReadout cost={renderService.frameCost} />

          <ViewportControls onZoomIn={onZoomIn} onZoomOut={onZoomOut} onReset={onReset} />
        </>
      ) : null}
    </div>
  );
}
