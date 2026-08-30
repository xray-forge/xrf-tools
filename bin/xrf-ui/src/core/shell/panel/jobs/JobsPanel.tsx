import { Box, Divider, Typography } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement, useCallback } from "react";

import { JobDescription } from "@/core/bindings/types/xrf-app";
import { useJobsListing } from "@/core/jobs/lib/use-jobs-listing";
import { JobsService } from "@/core/jobs/services/jobs";
import { JobRow } from "@/core/shell/panel/jobs/JobRow";

/**
 * What the backend is doing, and what it recently finished.
 *
 * Read straight from the backend rather than from `JobsService`, because the two answer different questions: the
 * service knows the runs this window started or adopted, and this is the only surface that can also see the ones that
 * have already ended.
 *
 * Running first, then the last few that finished, which is the order the backend already answers in.
 */
export function JobsPanel(): ReactElement {
  const jobsService: JobsService = useInjection(JobsService);
  const listed: Array<JobDescription> = useJobsListing();

  const onCancel = useCallback((id: string) => jobsService.cancel(id), [jobsService]);

  return (
    <Box sx={{ display: "flex", flexDirection: "column", minHeight: 0, height: "100%" }}>
      <Box sx={{ paddingX: 1.5, paddingY: 1 }}>
        <Typography variant={"subtitle2"}>Jobs</Typography>
      </Box>

      <Divider />

      <Box sx={{ flexGrow: 1, minHeight: 0, overflowY: "auto" }}>
        {listed.length ? (
          listed.map((job: JobDescription) => <JobRow key={job.id} job={job} onCancel={onCancel} />)
        ) : (
          <Typography variant={"caption"} color={"text.secondary"} sx={{ display: "block", padding: 1.5 }}>
            Nothing is running, and nothing has finished recently.
          </Typography>
        )}
      </Box>
    </Box>
  );
}
