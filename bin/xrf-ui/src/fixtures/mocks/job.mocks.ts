import { EJobKind, JobDescription } from "@/core/ipc/types/xrf-app";

/** A job the backend lists: an archives pack running a second, nothing reported yet, unless told. */
export function mockJobDescription(patch: Partial<JobDescription> = {}): JobDescription {
  return {
    id: "b8f0",
    kind: EJobKind.ARCHIVES_PACK,
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
