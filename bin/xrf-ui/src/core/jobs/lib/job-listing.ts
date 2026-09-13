import { JobConclusion, JobDescription } from "@/core/ipc/types/xrf-app";

/** What every run of one kind came to. */
export interface IJobKindSummary {
  kind: string;
  runs: number;
  completed: number;
  cancelled: number;
  failed: number;
  /** Milliseconds across the runs of this kind that have ended. */
  duration: number;
  /** The longest single run, in milliseconds. */
  slowest: number;
}

/** One thing a running job holds, and what that would refuse. */
export interface IJobLease {
  key: string;
  /** Identity of the run holding it. */
  id: string;
  kind: string;
}

/**
 * Folds the listing into one row per kind.
 *
 * @param jobs - Every job the listing described.
 * @returns One summary per kind seen, in the order the kinds first appear.
 */
export function summarizeJobKinds(jobs: ReadonlyArray<JobDescription>): Array<IJobKindSummary> {
  const summaries: Map<string, IJobKindSummary> = new Map();

  for (const job of jobs) {
    const summary: IJobKindSummary = summaries.get(job.kind) ?? {
      kind: job.kind,
      runs: 0,
      completed: 0,
      cancelled: 0,
      failed: 0,
      duration: 0,
      slowest: 0,
    };

    summary.runs += 1;

    if (job.conclusion) {
      summary[conclusionField(job.conclusion)] += 1;
      summary.duration += job.duration;
      summary.slowest = Math.max(summary.slowest, job.duration);
    }

    summaries.set(job.kind, summary);
  }

  return Array.from(summaries.values());
}

/**
 * What the running jobs hold exclusively.
 *
 * @param jobs - Every job the listing described.
 * @returns One entry per held key, in listing order.
 */
export function listHeldLeases(jobs: ReadonlyArray<JobDescription>): Array<IJobLease> {
  return jobs
    .filter((job: JobDescription) => job.conclusion === null)
    .flatMap((job: JobDescription) => job.leaseKeys.map((key: string) => ({ key, id: job.id, kind: job.kind })));
}

/**
 * @param conclusion - How a run ended.
 * @returns The summary field counting that ending.
 */
function conclusionField(conclusion: JobConclusion): "completed" | "cancelled" | "failed" {
  switch (conclusion) {
    case "completed":
      return "completed";
    case "cancelled":
      return "cancelled";
    case "failed":
      return "failed";
  }
}
