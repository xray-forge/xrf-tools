import { Nullable } from "@xrf/types";
import { Fragment, ReactElement, ReactNode } from "react";

import { ERenderAntialiasing, RenderFrameReport, RenderPassCost } from "@/core/ipc/types/xrf-renderer";
import { RenderViewportOverlay, TRenderOverlayCorner } from "@/core/render/components/overlay/RenderViewportOverlay";
import { describeRenderAntialiasing } from "@/core/render/lib/features";
import { groupRenderPassCosts, IRenderPassGroup } from "@/core/render/lib/frame/render-pass-groups";
import { abbreviateRenderBackend } from "@/core/render/lib/settings/render-backend-choice";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatMilliseconds } from "@/lib/format/duration";
import { formatCount } from "@/lib/format/number";

/** The antialiasing modes that resolve the frame temporally, upscaling it to the output as they do. */
const TEMPORAL_ANTIALIASING: ReadonlySet<string> = new Set<string>([
  ERenderAntialiasing.TAA,
  ERenderAntialiasing.FSR2,
  ERenderAntialiasing.DLSS,
]);

/**
 * Where the scene is drawn or resolved at another size than the viewport's: what takes it from there and to which
 * size, the temporal resolve to the output, else FSR 1 to the viewport, the target left out where it draws at it;
 * none at the viewport's own size.
 */
function toScalingLine(report: RenderFrameReport): Nullable<string> {
  const isTemporal: boolean = TEMPORAL_ANTIALIASING.has(report.antialiasing);
  const target: string = isTemporal
    ? `${report.outputWidth} × ${report.outputHeight}`
    : `${report.width} × ${report.height}`;
  const isScaled: boolean =
    report.renderWidth > 0 &&
    (report.renderWidth !== report.width ||
      report.renderHeight !== report.height ||
      (isTemporal && report.outputWidth !== report.width));

  if (!isScaled) {
    return null;
  }

  const name: string = isTemporal ? describeRenderAntialiasing(report.antialiasing) : "FSR 1";
  const source: string = `${report.renderWidth} × ${report.renderHeight}`;

  return source === target ? `${name} ${source}` : `${name} ${source} → ${target}`;
}

/** The triangles drawn, after the graphics API's tag once a frame names the GPU it was drawn on. */
function toDrawnLine(report: RenderFrameReport): string {
  const triangles: string = `${formatCount(report.staticDraws.keptTriangles)} tris`;

  return report.adapter ? `${abbreviateRenderBackend(report.backend)} · ${triangles}` : triangles;
}

/** What every pass cost together. */
function toGpuTotal(passes: ReadonlyArray<RenderPassCost>): number {
  return passes.reduce((total: number, pass: RenderPassCost) => total + (pass.gpuTime ?? 0), 0);
}

export interface IRenderFrameReadoutProps extends BaseComponentProps {
  /**
   * What the last reported frame cost, with each pass's GPU time listed under the rest while the renderer times them, a
   * run of a family's passes summed into one row.
   */
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
      <div>{toDrawnLine(report)}</div>
      <div>{`${report.width} × ${report.height}`}</div>
      {toScalingLine(report) ? <div>{toScalingLine(report)}</div> : null}

      {children}

      {report.isGpuTimed && report.passes.length ? (
        <div data-testid={"render-frame-passes"} className={"mt-1 grid grid-cols-[auto_auto] gap-x-3"}>
          <div>GPU</div>
          <div className={"text-right tabular-nums"}>{`${toGpuTotal(report.passes).toFixed(2)} ms`}</div>

          {groupRenderPassCosts(report.passes).map((group: IRenderPassGroup, index: number) => (
            <Fragment key={`${index}-${group.name}`}>
              <div className={"opacity-70"}>{group.count > 1 ? `${group.name} ×${group.count}` : group.name}</div>
              <div className={"text-right tabular-nums opacity-70"}>{group.gpuTime.toFixed(2)}</div>
            </Fragment>
          ))}
        </div>
      ) : null}
    </RenderViewportOverlay>
  );
}
