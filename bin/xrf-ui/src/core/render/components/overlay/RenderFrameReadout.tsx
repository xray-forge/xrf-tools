import { IRendererPassCost, IRendererPassTimings, IRenderFrameCost } from "@xrf/renderer";
import { Fragment, ReactElement, ReactNode } from "react";

import { RenderViewportOverlay, TRenderOverlayCorner } from "@/core/render/components/overlay/RenderViewportOverlay";
import { BaseComponentProps } from "@/lib/dom/element-types";

/** The buffer's size, and the scene's as drawn where TAA upscales it from less. */
function toSizeLine(cost: IRenderFrameCost): string {
  const drawn: string = `${cost.drawnWidth} × ${cost.drawnHeight}`;
  const isUpscaled: boolean =
    cost.renderedWidth > 0 && (cost.renderedWidth !== cost.drawnWidth || cost.renderedHeight !== cost.drawnHeight);

  return isUpscaled ? `${drawn} from ${cost.renderedWidth} × ${cost.renderedHeight}` : drawn;
}

/** What every pass cost together. */
function toGpuTotal(passes: ReadonlyArray<IRendererPassCost>): number {
  return passes.reduce((total: number, pass: IRendererPassCost) => total + pass.gpuTime, 0);
}

export interface IRenderFrameReadoutProps extends BaseComponentProps {
  /** What the last reported frame cost. */
  cost: IRenderFrameCost;
  /** What each pass of it cost on the GPU, listed while passes are timed. */
  timings?: IRendererPassTimings;
  corner?: TRenderOverlayCorner;
  /** Whether each pass's GPU time is listed, under what the frame cost as a whole. */
  isAdvanced?: boolean;
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
  timings,
  corner = "top-left",
  isAdvanced = true,
  children,
}: IRenderFrameReadoutProps): ReactElement {
  return (
    <RenderViewportOverlay data-testid={dataTestId} id={id} className={className} corner={corner}>
      <div>{`${cost.framesPerSecond.toFixed(0)} fps · ${cost.frameTime.toFixed(1)} ms`}</div>
      <div>{`${cost.draws} draws · ${cost.triangles.toLocaleString()} tris`}</div>
      <div>{toSizeLine(cost)}</div>

      {children}

      {isAdvanced && timings?.isGpuTimed && timings.passes.length ? (
        <div data-testid={"render-frame-passes"} className={"mt-1 grid grid-cols-[auto_auto] gap-x-3"}>
          <div>GPU</div>
          <div className={"text-right tabular-nums"}>{`${toGpuTotal(timings.passes).toFixed(2)} ms`}</div>

          {timings.passes.map((pass: IRendererPassCost) => (
            <Fragment key={pass.name}>
              <div className={"opacity-70"}>{pass.name}</div>
              <div className={"text-right tabular-nums opacity-70"}>{pass.gpuTime.toFixed(2)}</div>
            </Fragment>
          ))}
        </div>
      ) : null}
    </RenderViewportOverlay>
  );
}
