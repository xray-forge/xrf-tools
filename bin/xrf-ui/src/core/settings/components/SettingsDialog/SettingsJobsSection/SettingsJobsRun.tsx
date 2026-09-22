import { Collapse, Divider, Typography } from "@mui/material";
import { Nullable } from "@xrf/types";
import { format } from "date-fns";
import { ReactElement, useState } from "react";

import { JobDescription } from "@/core/ipc/types/xrf-app";
import { formatProgressRate, formatProgressUnits } from "@/core/jobs/lib/progress-format";
import { IJobPhase, IJobProfile } from "@/core/jobs/metrics";
import { cn } from "@/lib/dom/dom-name";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";

import { describeJobOutcome, toPhaseShare } from "./SettingsJobsSection.utils";

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
    <div data-testid={dataTestId} className={className} id={id}>
      <div
        className={cn("flex items-center gap-4 py-1.5", profile ? "cursor-pointer" : "cursor-default")}
        onClick={() => setOpen(profile ? !isOpen : false)}
      >
        <Typography className={"w-16 shrink-0 text-text-secondary"} variant={"caption"}>
          {format(job.startedAt, "HH:mm:ss")}
        </Typography>

        <div className={"min-w-0 grow"}>
          <Typography className={"monospace wrap-anywhere"}>{job.kind}</Typography>

          {job.error ? (
            <Typography className={"block wrap-anywhere text-error"} variant={"caption"}>
              {job.error}
            </Typography>
          ) : null}
        </div>

        <Typography className={"shrink-0"} variant={"caption"} sx={{ color: outcome.color }}>
          {outcome.label}
        </Typography>

        <Typography className={"w-16 shrink-0 text-right"} variant={"body2"}>
          {formatDuration(job.duration)}
        </Typography>
      </div>

      <Collapse in={isOpen} unmountOnExit>
        <div className={"pb-2 pl-16"}>
          <Divider className={"mb-2"} />

          {profile?.phases.length ? (
            profile.phases.map((phase: IJobPhase) => {
              const share: Nullable<number> = toPhaseShare(phase.duration, sampled);

              return (
                <div key={phase.id} className={"flex gap-4 py-0.5"}>
                  <Typography className={"monospace min-w-0 grow"}>{phase.label ?? phase.id}</Typography>

                  <Typography className={"w-24 shrink-0 text-text-secondary"} variant={"caption"}>
                    {formatProgressUnits(phase.completed, phase.unit)}
                  </Typography>

                  <Typography className={"w-16 shrink-0 text-right"} variant={"caption"}>
                    {formatDuration(phase.duration)}
                  </Typography>

                  <Typography className={"w-11 shrink-0 text-right text-text-secondary"} variant={"caption"}>
                    {share === null ? "—" : `${Math.round(share)}%`}
                  </Typography>
                </div>
              );
            })
          ) : (
            <Typography className={"text-text-secondary"} variant={"caption"}>
              No phases were sampled.
            </Typography>
          )}

          <Typography className={"mt-2 block text-text-secondary"} variant={"caption"}>
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
        </div>
      </Collapse>
    </div>
  );
}
