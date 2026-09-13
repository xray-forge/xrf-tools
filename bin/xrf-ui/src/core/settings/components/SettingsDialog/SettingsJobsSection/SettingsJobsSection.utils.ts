import { JobConclusion, JobDescription } from "@/core/bindings/types/xrf-app";
import { Nullable } from "@/lib/types/general";

/** How a conclusion is painted, in the palette the rest of the application uses for the same meanings. */
export type TJobOutcomeColor = "success.main" | "warning.main" | "error.main" | "text.secondary";

const OUTCOME_COLORS: Record<JobConclusion, TJobOutcomeColor> = {
  completed: "success.main",
  cancelled: "warning.main",
  failed: "error.main",
};

/**
 * What to call how a run ended.
 *
 * @param job - Job as the listing describes it.
 * @returns Its outcome and the colour to paint it.
 */
export function describeJobOutcome(job: JobDescription): { label: string; color: TJobOutcomeColor } {
  if (job.conclusion === null) {
    return { label: job.isCancelRequested ? "stopping" : "running", color: "text.secondary" };
  }

  return { label: job.conclusion, color: OUTCOME_COLORS[job.conclusion] };
}

/**
 * The share of a run one phase took.
 *
 * @param duration - Milliseconds the phase held.
 * @param total - Milliseconds across every sampled phase.
 * @returns The share as a percentage, or null when nothing was sampled to compare it against.
 */
export function toPhaseShare(duration: number, total: number): Nullable<number> {
  return total > 0 ? (duration / total) * 100 : null;
}
