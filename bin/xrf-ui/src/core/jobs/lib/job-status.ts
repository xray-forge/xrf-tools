import { JobConclusion, JobDescription } from "@/core/ipc/types/xrf-app";

/** How a job's status is painted, in the palette the rest of the application uses for the same meanings. */
export type TJobStatusColor = "success.main" | "error.main" | "text.secondary";

/** A stopped run is neutral, as every notice reports one: nothing went wrong, and what it wrote stands. */
const CONCLUSION_COLORS: Record<JobConclusion, TJobStatusColor> = {
  completed: "success.main",
  cancelled: "text.secondary",
  failed: "error.main",
};

/** What a job's status reads as, and how it is painted. */
export interface IJobStatus {
  label: string;
  color: TJobStatusColor;
}

/**
 * What to call where a job is: running, stopping, or how it ended.
 *
 * @param job - Job as the listing describes it.
 * @returns Its status and the colour to paint it.
 */
export function describeJobStatus(job: JobDescription): IJobStatus {
  if (job.conclusion === null) {
    return { label: job.isCancelRequested ? "stopping" : "running", color: "text.secondary" };
  }

  return { label: job.conclusion, color: CONCLUSION_COLORS[job.conclusion] };
}
