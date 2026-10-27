import { formatDuration } from "@xrf/format";
import { ProgressLevel } from "@xrf/ipc/types/xrf-job";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { Button } from "@xrf/ui/material/button";
import { LinearProgress } from "@xrf/ui/material/progress";
import { Typography } from "@xrf/ui/material/typography";
import { cn } from "@xrf/ui/theme";
import { ReactElement } from "react";

import { IJobState } from "@/core/jobs/lib";
import { describeActiveProgress, RENDERED_PROGRESS_LEVELS } from "@/core/jobs/lib/progress-format";

import { JobProgressLevel } from "./JobProgressLevel";

interface IJobProgressViewProps extends BaseComponentProps {
  job: IJobState;
  /** Omitted where a job is only being watched, such as a listing of somebody else's run. */
  onCancel?: (id: string) => void;
}

/**
 * What a running job looks like while it runs: its levels as bars, what it is on, and how to stop it.
 *
 * Renders before the first snapshot arrives, because the job exists from the moment it was started and a blank space
 * there would read as the control having done nothing.
 */
export function JobProgressView({
  "data-testid": dataTestId,
  id,
  className,
  job,
  onCancel,
}: IJobProgressViewProps): ReactElement {
  const levels: Array<ProgressLevel> = job.progress?.levels ?? [];
  const detail: string = describeActiveProgress(job.progress);

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex w-full flex-col gap-2", className)}>
      {levels.length ? (
        levels
          .slice(0, RENDERED_PROGRESS_LEVELS)
          .map((level: ProgressLevel) => <JobProgressLevel key={level.id} level={level} />)
      ) : (
        <LinearProgress variant={"indeterminate"} />
      )}

      <div className={"flex items-center justify-between gap-2"}>
        <div className={"min-w-0"}>
          <Typography variant={"caption"} color={"text.secondary"} noWrap={true} title={detail}>
            {detail}
          </Typography>
        </div>

        <div className={"flex shrink-0 items-center gap-2"}>
          <Typography variant={"caption"} color={"text.secondary"}>
            {formatDuration(job.progress?.duration ?? 0)}
          </Typography>

          {onCancel ? (
            <Button size={"small"} color={"inherit"} disabled={job.isCancelRequested} onClick={() => onCancel(job.id)}>
              {job.isCancelRequested ? "Stopping" : "Cancel"}
            </Button>
          ) : null}
        </div>
      </div>
    </div>
  );
}
