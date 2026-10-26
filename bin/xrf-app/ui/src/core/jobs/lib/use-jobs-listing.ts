import { jobsCommands } from "@xrf/ipc/commands/jobs";
import { JobDescription } from "@xrf/ipc/types/xrf-app-core";

import { usePolledValue } from "@/lib/react/use-polled-value";
import { EMPTY_ARRAY } from "@/lib/types/array";

/**
 * How often the listing asks the backend what it is doing.
 */
const LISTING_POLL_INTERVAL: number = 1000;

/**
 * Every job the backend is running, and the last few it finished.
 *
 * @returns The listing, newest state each poll, empty until the first answer arrives.
 */
export function useJobsListing(): ReadonlyArray<JobDescription> {
  return usePolledValue(async () => jobsCommands.list(), LISTING_POLL_INTERVAL) ?? EMPTY_ARRAY;
}
