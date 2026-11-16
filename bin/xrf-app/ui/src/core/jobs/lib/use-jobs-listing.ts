import { useInjection } from "@wirestate/react";
import { EMPTY_ARRAY } from "@xrf/collections";
import { JobDescription } from "@xrf/ipc/types/xrf-app-core";

import { JobsService } from "@/core/jobs/services/jobs.service";
import { usePolledCommand } from "@/core/operations/lib/use-polled-command";

/**
 * Every job the backend is running, and the last few it finished, sharing one poll with every other reader.
 *
 * @returns The listing, newest state each poll, empty until the first answer arrives.
 */
export function useJobsListing(): ReadonlyArray<JobDescription> {
  const jobsService: JobsService = useInjection(JobsService);

  return usePolledCommand(jobsService.listing) ?? EMPTY_ARRAY;
}
