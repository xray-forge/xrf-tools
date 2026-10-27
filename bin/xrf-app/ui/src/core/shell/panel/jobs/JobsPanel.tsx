import { useInjection } from "@wirestate/react";
import { JobDescription } from "@xrf/ipc/types/xrf-app-core";
import { Typography } from "@xrf/ui/material/typography";
import { ReactElement, useCallback } from "react";

import { useJobsListing } from "@/core/jobs/lib/use-jobs-listing";
import { JobsService } from "@/core/jobs/services/jobs.service";
import { EditorPanel } from "@/core/shell/editor/EditorPanel";
import { JobRow } from "@/core/shell/panel/jobs/JobRow";

/**
 * What the backend is doing, and what it recently finished.
 */
export function JobsPanel(): ReactElement {
  const jobsService: JobsService = useInjection(JobsService);
  const listed: ReadonlyArray<JobDescription> = useJobsListing();

  const onCancel = useCallback((id: string) => jobsService.cancel(id), [jobsService]);

  return (
    <EditorPanel className={"h-full"} title={"Jobs"}>
      <div className={"h-full overflow-y-auto p-4"}>
        {listed.length ? (
          listed.map((job: JobDescription) => <JobRow key={job.id} job={job} onCancel={onCancel} />)
        ) : (
          <Typography className={"leading-panel text-text-secondary"} variant={"body2"}>
            Nothing is running, and nothing has finished recently.
          </Typography>
        )}
      </div>
    </EditorPanel>
  );
}
