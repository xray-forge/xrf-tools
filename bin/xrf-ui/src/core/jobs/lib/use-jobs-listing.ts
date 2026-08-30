import { useEffect, useState } from "react";

import { jobsCommands } from "@/core/bindings/commands/jobs";
import { JobDescription } from "@/core/bindings/types/xrf-app";

/**
 * How often the listing asks the backend what it is doing.
 */
const LISTING_POLL_INTERVAL: number = 1000;

/**
 * Every job the backend is running, and the last few it finished.
 *
 * @returns The listing, newest state each poll, empty until the first answer arrives.
 */
export function useJobsListing(): Array<JobDescription> {
  const [listed, setListed] = useState<Array<JobDescription>>([]);

  useEffect(() => {
    let isWatching: boolean = true;

    async function read(): Promise<void> {
      try {
        const answer: Array<JobDescription> = await jobsCommands.list();

        // The reader may have gone away while the request was in flight, and setting state then is a write into a
        // component that no longer exists.
        if (isWatching) {
          setListed(answer);
        }
      } catch {
        // Left to the next tick. Reporting it here would put an error in a panel whose whole job is to report on
        // other things.
      }
    }

    void read();

    const timer: ReturnType<typeof setInterval> = setInterval(() => void read(), LISTING_POLL_INTERVAL);

    return () => {
      isWatching = false;

      clearInterval(timer);
    };
  }, []);

  return listed;
}
