import { Box, Button, Divider, Stack, Typography } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { JobDescription } from "@/core/ipc/types/xrf-app";
import { IJobKindSummary, IJobLease, listHeldLeases, summarizeJobKinds } from "@/core/jobs/lib/job-listing";
import { useJobsListing } from "@/core/jobs/lib/use-jobs-listing";
import { IJobProfile, JOB_PROFILES } from "@/core/jobs/metrics";
import { SettingsSection } from "@/core/settings/components/SettingsSection";
import { SettingsStat } from "@/core/settings/components/SettingsStat";
import { MONOSPACE } from "@/core/theme/tokens";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatDuration } from "@/lib/format/duration";
import { useForceUpdate } from "@/lib/react";
import { Callable } from "@/lib/types/general";

import { SettingsJobsRun } from "./SettingsJobsRun";

/**
 * What the backend has been asked to do, and how those runs went.
 */
export function SettingsJobsSection({
  "data-testid": dataTestId = "settings-jobs-section",
  className,
  id,
}: BaseComponentProps): ReactElement {
  const listed: Array<JobDescription> = useJobsListing();

  const forceUpdate: Callable = useForceUpdate();

  const summaries: Array<IJobKindSummary> = summarizeJobKinds(listed);
  const leases: Array<IJobLease> = listHeldLeases(listed);
  const running: number = listed.filter((it: JobDescription) => it.conclusion === null).length;
  const profiles: Map<string, IJobProfile> = new Map(
    JOB_PROFILES.list().map((it: IJobProfile) => [it.id, it] as const)
  );

  const onClear = useCallback(() => {
    JOB_PROFILES.reset();
    forceUpdate();
  }, [forceUpdate]);

  return (
    <Box
      data-testid={dataTestId}
      id={id}
      className={className}
      sx={{ display: "flex", flexDirection: "column", gap: 3 }}
    >
      <SettingsSection
        title={"Runs"}
        description={"What the backend is running and the last it finished."}
        fact={listed.length === 1 ? "1 run" : `${listed.length} runs`}
      >
        <Box sx={{ display: "flex", flexWrap: "wrap", gap: 2, marginTop: 1 }}>
          <SettingsStat label={"Running"} value={String(running)} />
          <SettingsStat label={"Finished"} value={String(listed.length - running)} />
          <SettingsStat label={"Kinds"} value={String(summaries.length)} />
          <SettingsStat label={"Held leases"} value={String(leases.length)} />
        </Box>

        {listed.length ? (
          <Stack divider={<Divider flexItem />} sx={{ marginTop: 1 }}>
            {listed.map((job: JobDescription) => (
              <SettingsJobsRun key={job.id} job={job} profile={profiles.get(job.id) ?? null} />
            ))}
          </Stack>
        ) : (
          <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block", marginTop: 1 }}>
            Nothing has run yet.
          </Typography>
        )}
      </SettingsSection>

      <SettingsSection title={"By kind"} description={"Every kind of work in the listing, and what its runs came to."}>
        <Stack divider={<Divider flexItem />} sx={{ marginTop: 1 }}>
          {summaries.map((it: IJobKindSummary) => (
            <Box key={it.kind} sx={{ alignItems: "center", display: "flex", gap: 2, paddingY: 0.75 }}>
              <Typography sx={{ ...MONOSPACE, flexGrow: 1, minWidth: 0, overflowWrap: "anywhere" }}>
                {it.kind}
              </Typography>

              <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
                {[
                  it.completed ? `${it.completed} ok` : null,
                  it.cancelled ? `${it.cancelled} cancelled` : null,
                  it.failed ? `${it.failed} failed` : null,
                ]
                  .filter(Boolean)
                  .join(" · ")}
              </Typography>

              <Typography variant={"body2"} sx={{ flexShrink: 0, textAlign: "right", width: 76 }}>
                {it.runs}
              </Typography>

              <Typography variant={"body2"} sx={{ flexShrink: 0, textAlign: "right", width: 76 }}>
                {formatDuration(it.slowest)}
              </Typography>
            </Box>
          ))}
        </Stack>

        {summaries.length ? null : (
          <Typography variant={"caption"} sx={{ color: "text.secondary", display: "block", marginTop: 1 }}>
            Nothing has run yet.
          </Typography>
        )}
      </SettingsSection>

      {leases.length ? (
        <SettingsSection
          title={"Held exclusively"}
          description={"What the running jobs hold, which is what a refused start would be pointing at."}
        >
          <Stack sx={{ marginTop: 1 }}>
            {leases.map((it: IJobLease) => (
              <Box key={`${it.id}:${it.key}`} sx={{ display: "flex", gap: 2, paddingY: 0.25 }}>
                <Typography sx={{ ...MONOSPACE, flexGrow: 1, minWidth: 0, overflowWrap: "anywhere" }}>
                  {it.key}
                </Typography>

                <Typography variant={"caption"} sx={{ color: "text.secondary", flexShrink: 0 }}>
                  {it.kind}
                </Typography>
              </Box>
            ))}
          </Stack>
        </SettingsSection>
      ) : null}

      <Box sx={{ display: "flex", flexDirection: "column", gap: 2 }}>
        <Divider />

        <Box>
          <Button color={"error"} size={"small"} variant={"outlined"} onClick={onClear}>
            Clear
          </Button>
        </Box>
      </Box>
    </Box>
  );
}
