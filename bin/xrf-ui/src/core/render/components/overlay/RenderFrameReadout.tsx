import { Fragment, ReactElement, ReactNode } from "react";

import { RenderFrameReport, RenderPassCost } from "@/core/ipc/types/xrf-renderer";
import { RenderViewportOverlay, TRenderOverlayCorner } from "@/core/render/components/overlay/RenderViewportOverlay";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatMilliseconds } from "@/lib/format/duration";
import { formatCount } from "@/lib/format/number";

/** The buffer's size, and the scene's as drawn where it is upscaled from less. */
function toSizeLine(report: RenderFrameReport): string {
  const drawn: string = `${report.width} × ${report.height}`;
  const isUpscaled: boolean =
    report.renderWidth > 0 && (report.renderWidth !== report.width || report.renderHeight !== report.height);

  return isUpscaled ? `${drawn} from ${report.renderWidth} × ${report.renderHeight}` : drawn;
}

/** What every pass cost together. */
function toGpuTotal(passes: ReadonlyArray<RenderPassCost>): number {
  return passes.reduce((total: number, pass: RenderPassCost) => total + (pass.gpuTime ?? 0), 0);
}

export interface IRenderFrameReadoutProps extends BaseComponentProps {
  /** What the last reported frame cost, with each pass's GPU time listed under the rest while the renderer times them. */
  report: RenderFrameReport;
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
  report,
  corner = "top-left",
  children,
}: IRenderFrameReadoutProps): ReactElement {
  return (
    <RenderViewportOverlay data-testid={dataTestId} id={id} className={className} corner={corner}>
      <div>{`${(report.framesPerSecond ?? 0).toFixed(0)} fps · ${formatMilliseconds(report.frameTime ?? 0)}`}</div>
      <div>{`${formatCount(report.staticDraws.commands)} draws · ${formatCount(report.staticDraws.keptTriangles)} tris`}</div>
      <div>{toSizeLine(report)}</div>

      {children}

      {report.isGpuTimed && report.passes.length ? (
        <div data-testid={"render-frame-passes"} className={"mt-1 grid grid-cols-[auto_auto] gap-x-3"}>
          <div>GPU</div>
          <div className={"text-right tabular-nums"}>{`${toGpuTotal(report.passes).toFixed(2)} ms`}</div>

          {report.passes.map((pass: RenderPassCost) => (
            <Fragment key={pass.name}>
              <div className={"opacity-70"}>{pass.name}</div>
              <div className={"text-right tabular-nums opacity-70"}>{(pass.gpuTime ?? 0).toFixed(2)}</div>
            </Fragment>
          ))}
        </div>
      ) : null}
    </RenderViewportOverlay>
  );
}
