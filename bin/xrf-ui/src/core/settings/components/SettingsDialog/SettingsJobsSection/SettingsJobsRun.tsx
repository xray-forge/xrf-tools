import { Box, Collapse, Divider, Typography } from "@mui/material";
import { format } from "date-fns";
import { ReactElement, useState } from "react";

import { JobDescription } from "@/core/bindings/types/xrf-app";
import { formatProgressRate, formatProgressUnits } from "@/core/jobs/lib/progress-format";
import { IJobPhase, IJobProfile } from "@/core/jobs/metrics";
import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { Nullable } from "@/lib/types/general";

import { describeJobOutcome, toPhaseShare } from "./SettingsJobsSection.utils";

/** Room the timestamp and duration columns keep, so the rows line up. */
const STAMP_WIDTH: number = 64;

export interface ISettingsJobsRunProps extends BaseComponentProps {
  job: JobDescription;
  profile: Nullable<IJobProfile>;
}

/**
 * One run, and what this window saw of how it got there.
 */
export function SettingsJobsRun({
  "data-testid": dataTestId = "settings-jobs-run",
  className,
  id,
  job,
  profile,
}: ISettingsJobsRunProps): ReactElement {
  const [isOpen, setOpen] = useState<boolean>(false);

  const outcome = describeJobOutcome(job);
  const sampled: number = (profile?.phases ?? []).reduce((total: number, it: IJobPhase) => total + it.duration, 0);

  return (
    <Box data-testid={dataTestId} className={className} id={id}>
      <Box
        sx={{ alignItems: "center", cursor: profile ? "pointer" : "default", display: "flex", gap: 2, paddingY: 0.75 }}
        onClick={() => setOpen(profile ? !isOpen : false)}
      >
        <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0, width: STAMP_WIDTH }}>
          {format(job.startedAt, "HH:mm:ss")}
        </Typography>

        <Box sx={{ flexGrow: 1, minWidth: 0 }}>
          <Typography sx={{ ...MONOSPACE, overflowWrap: "anywhere" }}>{job.kind}</Typography>

          {job.error ? (
            <Typography variant={"caption"} sx={{ color: "error.main", display: "block", overflowWrap: "anywhere" }}>
              {job.error}
            </Typography>
          ) : null}
        </Box>

        <Typography variant={"caption"} sx={{ color: outcome.color, flexShrink: 0 }}>
          {outcome.label}
        </Typography>

        <Typography variant={"body2"} sx={{ flexShrink: 0, textAlign: "right", width: STAMP_WIDTH }}>
          {formatDuration(job.duration)}
        </Typography>
      </Box>

      <Collapse in={isOpen} unmountOnExit>
        <Box sx={{ paddingBottom: 1, paddingLeft: STAMP_WIDTH / 8 }}>
          <Divider sx={{ marginBottom: 1 }} />

          {profile?.phases.length ? (
            profile.phases.map((phase: IJobPhase) => {
              const share: Nullable<number> = toPhaseShare(phase.duration, sampled);

              return (
                <Box key={phase.id} sx={{ display: "flex", gap: 2, paddingY: 0.25 }}>
                  <Typography sx={{ ...MONOSPACE, flexGrow: 1, minWidth: 0 }}>{phase.label ?? phase.id}</Typography>

                  <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0, width: 96 }}>
                    {formatProgressUnits(phase.completed, phase.unit)}
                  </Typography>

                  <Typography variant={"caption"} sx={{ flexShrink: 0, textAlign: "right", width: STAMP_WIDTH }}>
                    {formatDuration(phase.duration)}
                  </Typography>

                  <Typography
                    variant={"caption"}
                    sx={{ color: "text.secondary", flexShrink: 0, textAlign: "right", width: 44 }}
                  >
                    {share === null ? "—" : `${Math.round(share)}%`}
                  </Typography>
                </Box>
              );
            })
          ) : (
            <Typography variant={"caption"} sx={{ color: "text.secondary" }}>
              No phases were sampled.
            </Typography>
          )}

          <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block", marginTop: 1 }}>
            {[
              `${profile?.samples ?? 0} reports`,
              profile?.peakRate && profile.peakUnit
                ? `peak ${formatProgressRate(profile.peakRate, profile.peakUnit)}`
                : null,
              profile?.longestStall ? `longest stall ${formatDuration(profile.longestStall)}` : null,
              profile?.isPartial ? "joined after this window reloaded, so earlier phases are missing" : null,
            ]
              .filter(Boolean)
              .join(" · ")}
          </Typography>
        </Box>
      </Collapse>
    </Box>
  );
}
