import { formatDuration } from "@xrf/format";
import { JobDescription } from "@xrf/ipc/types/xrf-app-core";
import { Callable } from "@xrf/types";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { DetailSection } from "@xrf/ui/layout/detail-section";
import { Button } from "@xrf/ui/material/button";
import { Divider } from "@xrf/ui/material/divider";
import { Stack } from "@xrf/ui/material/stack";
import { Typography } from "@xrf/ui/material/typography";
import { StatFigure } from "@xrf/ui/stats";
import { cn } from "@xrf/ui/theme";
import { ReactElement, useCallback } from "react";

import { IJobKindSummary, IJobLease, listHeldLeases, summarizeJobKinds } from "@/core/jobs/lib/job-listing";
import { useJobsListing } from "@/core/jobs/lib/use-jobs-listing";
import { JOB_PROFILES } from "@/core/jobs/metrics";
import { useForceUpdate } from "@/lib/react/use-force-update";

import { SettingsJobsRun } from "./SettingsJobsRun";

/**
 * What the backend has been asked to do, and how those runs went.
 */
export function SettingsJobsSection({
  "data-testid": dataTestId = "settings-jobs-section",
  className,
  id,
}: BaseComponentProps): ReactElement {
  const listed: ReadonlyArray<JobDescription> = useJobsListing();

  const forceUpdate: Callable = useForceUpdate();

  const summaries: Array<IJobKindSummary> = summarizeJobKinds(listed);
  const leases: Array<IJobLease> = listHeldLeases(listed);
  const running: number = listed.filter((it: JobDescription) => it.conclusion === null).length;

  const onClear = useCallback(() => {
    JOB_PROFILES.reset();
    forceUpdate();
  }, [forceUpdate]);

  return (
    <div data-testid={dataTestId} id={id} className={cn("flex flex-col gap-6", className)}>
      <DetailSection
        title={"Runs"}
        description={"What the backend is running and the last it finished."}
        fact={listed.length === 1 ? "1 run" : `${listed.length} runs`}
      >
        <div className={"mt-2 flex flex-wrap gap-4"}>
          <StatFigure label={"Running"} value={String(running)} />
          <StatFigure label={"Finished"} value={String(listed.length - running)} />
          <StatFigure label={"Kinds"} value={String(summaries.length)} />
          <StatFigure label={"Held leases"} value={String(leases.length)} />
        </div>

        {listed.length ? (
          <Stack className={"mt-2"} divider={<Divider flexItem />}>
            {listed.map((job: JobDescription) => (
              <SettingsJobsRun key={job.id} job={job} profile={JOB_PROFILES.read(job.id)} />
            ))}
          </Stack>
        ) : (
          <Typography className={"mt-2 block text-text-secondary"} variant={"caption"}>
            Nothing has run yet.
          </Typography>
        )}
      </DetailSection>

      <DetailSection
        title={"By kind"}
        description={
          "Every kind of work in the listing, and what its runs came to. Durations include finished runs only."
        }
      >
        <Stack className={"mt-2"} divider={<Divider flexItem />}>
          {summaries.map((it: IJobKindSummary) => (
            <div key={it.kind} className={"flex items-center gap-4 py-1.5"}>
              <Typography className={"monospace min-w-0 grow wrap-anywhere"}>{it.kind}</Typography>

              <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
                {[
                  it.completed ? `${it.completed} ok` : null,
                  it.cancelled ? `${it.cancelled} cancelled` : null,
                  it.failed ? `${it.failed} failed` : null,
                ]
                  .filter(Boolean)
                  .join(" · ")}
              </Typography>

              <Typography className={"w-19 shrink-0 text-right"} variant={"body2"}>
                {it.runs}
              </Typography>

              <div className={"w-19 shrink-0 text-right"}>
                <Typography className={"text-text-secondary"} variant={"caption"}>
                  Total
                </Typography>
                <Typography variant={"body2"}>{formatDuration(it.duration)}</Typography>
              </div>

              <div className={"w-19 shrink-0 text-right"}>
                <Typography className={"text-text-secondary"} variant={"caption"}>
                  Slowest
                </Typography>
                <Typography variant={"body2"}>{formatDuration(it.slowest)}</Typography>
              </div>
            </div>
          ))}
        </Stack>

        {summaries.length ? null : (
          <Typography className={"mt-2 block text-text-secondary"} variant={"caption"}>
            Nothing has run yet.
          </Typography>
        )}
      </DetailSection>

      {leases.length ? (
        <DetailSection
          title={"Held exclusively"}
          description={"What the running jobs hold, which is what a refused start would be pointing at."}
        >
          <Stack className={"mt-2"}>
            {leases.map((it: IJobLease) => (
              <div key={`${it.id}:${it.key}`} className={"flex gap-4 py-0.5"}>
                <Typography className={"monospace min-w-0 grow wrap-anywhere"}>{it.key}</Typography>

                <Typography className={"shrink-0 text-text-secondary"} variant={"caption"}>
                  {it.kind}
                </Typography>
              </div>
            ))}
          </Stack>
        </DetailSection>
      ) : null}

      <div className={"flex flex-col gap-4"}>
        <Divider />

        <div>
          <Button color={"error"} size={"small"} variant={"outlined"} onClick={onClear}>
            Clear
          </Button>
        </div>
      </div>
    </div>
  );
}
