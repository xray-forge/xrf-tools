import { ReactElement, ReactNode } from "react";

import { RenderViewportOverlay, TRenderOverlayCorner } from "@/core/render/components/overlay/RenderViewportOverlay";
import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { ERenderThread } from "@/core/render/lib/frame/render-thread";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** What each thread is called where it is read rather than where it is chosen. */
const THREAD_LABELS: Record<ERenderThread, string> = {
  [ERenderThread.MAIN]: "main thread",
  [ERenderThread.WORKER]: "worker",
};

export interface IRenderFrameReadoutProps extends BaseComponentProps {
  /** What the last reported frame cost. */
  cost: IRenderFrameCost;
  /** Which thread drew it, which the picture itself never shows. */
  thread: ERenderThread;
  corner?: TRenderOverlayCorner;
  /** Anything the scene can say that a viewport cannot, drawn under the rest. */
  children?: ReactNode;
}

/**
 * What a viewport is costing, laid over the viewport that is costing it.
 */
export function RenderFrameReadout({
  "data-testid": dataTestId = "render-frame-readout",
  id,
  className,
  cost,
  thread,
  corner = "top-left",
  children,
}: IRenderFrameReadoutProps): ReactElement {
  return (
    <RenderViewportOverlay data-testid={dataTestId} id={id} className={className} corner={corner}>
      <div>{`${cost.framesPerSecond.toFixed(0)} fps · ${cost.frameTime.toFixed(1)} ms`}</div>
      <div>{`${cost.draws} draws · ${cost.triangles.toLocaleString()} tris`}</div>
      <div>{`${cost.drawnWidth} × ${cost.drawnHeight} · ${THREAD_LABELS[thread]}`}</div>

      {children}
    </RenderViewportOverlay>
  );
}
