import { LinearProgress, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { ProgressLevel } from "@/core/ipc/types/xrf-job";
import { formatProgressCounts, toProgressPercent } from "@/core/jobs/lib/progress-format";

export interface IJobProgressLevelProps {
  level: ProgressLevel;
}

/**
 * One level of a job's progress, as a labelled bar.
 */
export function JobProgressLevel({ level }: IJobProgressLevelProps): ReactElement {
  const percent: Nullable<number> = toProgressPercent(level);

  return (
    <div>
      <div className={"flex justify-between gap-2"}>
        <Typography variant={"caption"} color={"text.secondary"}>
          {level.label ?? level.id}
        </Typography>

        <Typography variant={"caption"} color={"text.secondary"}>
          {formatProgressCounts(level)}
        </Typography>
      </div>

      <LinearProgress variant={percent === null ? "indeterminate" : "determinate"} value={percent ?? undefined} />
    </div>
  );
}
