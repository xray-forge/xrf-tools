import { ERenderViewportJob, RenderViewportJob } from "@xrf/ipc/types/xrf-renderer";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { CircleIcon } from "@xrf/ui/icons/circle";
import { Tooltip } from "@xrf/ui/material/tooltip";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

/** What each long job is called and the colour its mark takes: red recording, amber a burst, blue a capture. */
const JOBS: Readonly<Record<ERenderViewportJob, { label: string; className: string }>> = {
  [ERenderViewportJob.RECORDING]: { label: "Recording", className: "text-error" },
  [ERenderViewportJob.BURST]: { label: "Drawing a burst", className: "text-warning" },
  [ERenderViewportJob.CAPTURE]: { label: "Capturing a level map or cubemap", className: "text-info" },
};

export interface IRenderViewportJobMarkProps extends BaseComponentProps {
  /** The long job the viewport runs. */
  job: RenderViewportJob;
}

/**
 * A dot by a viewport's frame rate while it runs a long job, coloured by the job and named by it.
 */
export function RenderViewportJobMark({
  "data-testid": dataTestId = "render-viewport-job-mark",
  id,
  className,
  job,
}: IRenderViewportJobMarkProps): ReactElement {
  const { label, className: color } = JOBS[job as ERenderViewportJob];

  return (
    <Tooltip title={label}>
      <span
        data-testid={dataTestId}
        id={id}
        className={cn("inline-flex", color, className)}
        role={"img"}
        aria-label={label}
      >
        <CircleIcon sx={{ fontSize: "0.6rem" }} />
      </span>
    </Tooltip>
  );
}
