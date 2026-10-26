import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { RenderFailureCover, RenderFrameReadout } from "@/core/render/components/overlay";
import { RenderSurface } from "@/core/render/components/RenderSurface";
import { DOLLY_STEP } from "@/core/render/lib/frame/render-dolly";
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

      {source.model && !renderService.failure ? (
        <>
          <RenderFrameReadout report={renderService.frame} />

          <ViewportControls onZoomIn={onZoomIn} onZoomOut={onZoomOut} onReset={onReset} />
        </>
      ) : null}

      <RenderFailureCover failure={renderService.failure} />
    </div>
  );
}
