import { JobConclusion } from "@/core/bindings/types/xrf-app";
import { findJobKind, IJobKindDescriptor } from "@/core/jobs/lib/job-kinds";
import { IJobNotice, IJobState } from "@/core/jobs/lib/jobs-types";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { Nullable } from "@/lib/types/general";

const ADOPTED_SEVERITIES: Record<JobConclusion, ENotificationSeverity> = {
  completed: ENotificationSeverity.SUCCESS,
  cancelled: ENotificationSeverity.INFO,
  failed: ENotificationSeverity.ERROR,
};

const ADOPTED_TITLES: Record<JobConclusion, string> = {
  completed: "Finished while the window was reloading",
  cancelled: "Stopped while the window was reloading",
  failed: "Failed while the window was reloading",
};

/**
 * What the notification centre says about a job this window adopted rather than started.
 *
 * Says what the work was rather than which kind string named it, so a record reads the way one from the tool itself
 * does. An unknown kind falls back to its spelling, which is all a build running against a newer backend has.
 *
 * @param job - The adopted job, as this window last saw it.
 * @param conclusion - How the backend says it ended, or null where it fell out of the retained listing first.
 * @param error - Why it failed, where the backend recorded a reason.
 * @returns What to record about it.
 */
export function describeAdoptedOutcome(
  job: IJobState,
  conclusion: Nullable<JobConclusion>,
  error: Nullable<string>
): IJobNotice {
  const described: Nullable<IJobKindDescriptor> = findJobKind(job.kind);
  const subject: string = described?.label ?? job.kind;

  if (!conclusion) {
    // The job left the retained listing before this window looked again, which takes twenty finished jobs. Reporting
    // it as completed would be a guess, and staying silent would lose a run the user started.
    return {
      details: [subject, "Its outcome is no longer recorded."].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Ended while the window was reloading",
    };
  }

  return {
    details: [subject, ...(error ? [error] : [])].join("\n"),
    severity: ADOPTED_SEVERITIES[conclusion],
    title: ADOPTED_TITLES[conclusion],
  };
}
