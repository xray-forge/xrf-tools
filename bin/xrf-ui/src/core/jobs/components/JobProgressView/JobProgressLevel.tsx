import { Box, LinearProgress, Typography } from "@mui/material";
import { ReactElement } from "react";

import { ProgressLevel } from "@/core/bindings/types/xrf-job";
import { formatProgressCounts, toProgressPercent } from "@/core/jobs/lib/progress-format";
import { Nullable } from "@/lib/types/general";

export interface IJobProgressLevelProps {
  level: ProgressLevel;
}

/**
 * One level of a job's progress, as a labelled bar.
 *
 * A level that cannot be counted renders an indeterminate bar beside its running count, which is the honest reading of
 * a phase that does not yet know its own size.
 */
export function JobProgressLevel({ level }: IJobProgressLevelProps): ReactElement {
  const percent: Nullable<number> = toProgressPercent(level);

  return (
    <Box>
      <Box sx={{ display: "flex", justifyContent: "space-between", gap: 1 }}>
        <Typography variant={"caption"} color={"text.secondary"}>
          {level.label ?? level.id}
        </Typography>

        <Typography variant={"caption"} color={"text.secondary"}>
          {formatProgressCounts(level)}
        </Typography>
      </Box>

      <LinearProgress
        variant={percent === null ? "indeterminate" : "determinate"}
        value={percent ?? undefined}
      />
    </Box>
  );
}
