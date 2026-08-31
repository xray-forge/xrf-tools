import { Box, Button, Chip, LinearProgress, Typography } from "@mui/material";
import { ReactElement } from "react";

import { JobConclusion, JobDescription } from "@/core/bindings/types/xrf-app";
import { ProgressLevel } from "@/core/bindings/types/xrf-job";
import { findJobKind, IJobKindDescriptor } from "@/core/jobs/lib";
import { formatProgressCounts, toProgressPercent } from "@/core/jobs/lib/progress-format";
import { formatDuration } from "@/lib/format/duration";
import { Nullable } from "@/lib/types/general";

const CONCLUSION_COLORS: Record<JobConclusion, string> = {
  completed: "success.main",
  cancelled: "text.secondary",
  failed: "error.main",
};

export interface IJobRowProps {
  job: JobDescription;
  onCancel: (id: string) => void;
}

/**
 * One job as the listing shows it, running or finished.
 *
 * Shows the identity, the leases, and the raw kind alongside the readable label — this is the surface for working out
 * why a run was refused or why one is still holding a destination, and every one of those answers is in a field a
 * person would otherwise have to read out of a log.
 */
export function JobRow({ job, onCancel }: IJobRowProps): ReactElement {
  const described: Nullable<IJobKindDescriptor> = findJobKind(job.kind);
  const active: Nullable<ProgressLevel> = job.progress?.levels.at(-1) ?? null;
  const percent: Nullable<number> = active ? toProgressPercent(active) : null;

  return (
    <Box sx={{ display: "flex", flexDirection: "column", gap: 0.5, paddingX: 1.5, paddingY: 1 }}>
      <Box sx={{ display: "flex", alignItems: "baseline", justifyContent: "space-between", gap: 1 }}>
        <Typography variant={"body2"} noWrap={true}>
          {described?.label ?? job.kind}
        </Typography>

        <Typography
          variant={"caption"}
          sx={{ color: job.conclusion ? CONCLUSION_COLORS[job.conclusion] : "text.secondary", flexShrink: 0 }}
        >
          {job.conclusion ?? (job.isCancelRequested ? "stopping" : "running")}
        </Typography>
      </Box>

      {job.conclusion ? null : (
        <LinearProgress variant={percent === null ? "indeterminate" : "determinate"} value={percent ?? undefined} />
      )}

      <Box sx={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: 1 }}>
        <Typography variant={"caption"} color={"text.secondary"} noWrap={true}>
          {active ? `${active.label ?? active.id} ${formatProgressCounts(active)}` : job.kind}
        </Typography>

        <Box sx={{ display: "flex", alignItems: "center", flexShrink: 0, gap: 1 }}>
          <Typography variant={"caption"} color={"text.secondary"}>
            {formatDuration(job.duration)}
          </Typography>

          {job.conclusion ? null : (
            <Button size={"small"} color={"inherit"} disabled={job.isCancelRequested} onClick={() => onCancel(job.id)}>
              {job.isCancelRequested ? "Stopping" : "Cancel"}
            </Button>
          )}
        </Box>
      </Box>

      {job.error ? (
        <Typography variant={"caption"} color={"error.main"}>
          {job.error}
        </Typography>
      ) : null}

      {/* The leases are why a second run was refused, so they are the first thing to look at when one was. */}
      {job.leaseKeys.length ? (
        <Box sx={{ display: "flex", flexWrap: "wrap", gap: 0.5 }}>
          {job.leaseKeys.map((key: string) => (
            <Chip key={key} size={"small"} variant={"outlined"} label={key} />
          ))}
        </Box>
      ) : null}
    </Box>
  );
}
