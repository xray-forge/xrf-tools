import { Box, Button, LinearProgress, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ProgressLevel } from "@/core/bindings/types/xrf-job";
import { JobProgressLevel } from "@/core/jobs/components/JobProgressView/JobProgressLevel";
import { IJobState } from "@/core/jobs/lib";
import { describeActiveProgress, RENDERED_PROGRESS_LEVELS } from "@/core/jobs/lib/progress-format";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";

export interface IJobProgressViewProps extends BaseComponentProps {
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
export function JobProgressView({ "data-testid": dataTestId, job, onCancel }: IJobProgressViewProps): ReactElement {
  const levels: Array<ProgressLevel> = job.progress?.levels ?? [];

  return (
    <Box data-testid={dataTestId} sx={{ display: "flex", flexDirection: "column", gap: 1, width: "100%" }}>
      {levels.length ? (
        levels
          .slice(0, RENDERED_PROGRESS_LEVELS)
          .map((level: ProgressLevel) => <JobProgressLevel key={level.id} level={level} />)
      ) : (
        <LinearProgress variant={"indeterminate"} />
      )}

      <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1 }}>
        <Box sx={{ minWidth: 0 }}>
          <Typography variant={"caption"} color={"text.secondary"} noWrap={true}>
            {describeActiveProgress(job.progress)}
          </Typography>
        </Box>

        <Box sx={{ display: "flex", alignItems: "center", flexShrink: 0, gap: 1 }}>
          <Typography variant={"caption"} color={"text.secondary"}>
            {formatDuration(job.progress?.duration ?? 0)}
          </Typography>

          {onCancel ? (
            <Button size={"small"} color={"inherit"} disabled={job.isCancelRequested} onClick={() => onCancel(job.id)}>
              {job.isCancelRequested ? "Stopping" : "Cancel"}
            </Button>
          ) : null}
        </Box>
      </Box>
    </Box>
  );
}
