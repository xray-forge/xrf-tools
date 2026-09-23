import { IRenderFrameCost } from "@xrf/renderer";
import { ReactElement, ReactNode } from "react";

import { RenderViewportOverlay, TRenderOverlayCorner } from "@/core/render/components/overlay/RenderViewportOverlay";
import { BaseComponentProps } from "@/lib/dom/element-types";

export interface IRenderFrameReadoutProps extends BaseComponentProps {
  /** What the last reported frame cost. */
  cost: IRenderFrameCost;
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
  corner = "top-left",
  children,
}: IRenderFrameReadoutProps): ReactElement {
  return (
    <RenderViewportOverlay data-testid={dataTestId} id={id} className={className} corner={corner}>
      <div>{`${cost.framesPerSecond.toFixed(0)} fps · ${cost.frameTime.toFixed(1)} ms`}</div>
      <div>{`${cost.draws} draws · ${cost.triangles.toLocaleString()} tris`}</div>
      <div>{`${cost.drawnWidth} × ${cost.drawnHeight}`}</div>

      {children}
    </RenderViewportOverlay>
  );
}
