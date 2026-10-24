import { ARCHIVES_JOB_PACK } from "@xrf/ipc/types/xrf-app-archives";
import { JobDescription } from "@xrf/ipc/types/xrf-app-core";

/** A job the backend lists: an archives pack running a second, nothing reported yet, unless told. */
export function mockJobDescription(patch: Partial<JobDescription> = {}): JobDescription {
  return {
    id: "b8f0",
    kind: ARCHIVES_JOB_PACK,
    leaseKeys: ["archives.pack:c:\\out|gamedata"],
    isCancelRequested: false,
    progress: null,
    conclusion: null,
    request: null,
    error: null,
    result: null,
    duration: 1000,
    ...patch,
  } as JobDescription;
}
