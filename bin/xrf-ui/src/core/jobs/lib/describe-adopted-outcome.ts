import { JobConclusion } from "@/core/bindings/types/xrf-app";
import { IJobState } from "@/core/jobs/lib/jobs-types";
import { ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
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
 * @param job - The adopted job, as this window last saw it.
 * @param conclusion - How the backend says it ended, or null where it fell out of the retained listing first.
 * @param error - Why it failed, where the backend recorded a reason.
 * @returns The notification to record.
 */
export function describeAdoptedOutcome(
  job: IJobState,
  conclusion: Nullable<JobConclusion>,
  error: Nullable<string>
): INotificationPayload {
  if (!conclusion) {
    // The job left the retained listing before this window looked again, which takes twenty finished jobs. Reporting
    // it as completed would be a guess, and staying silent would lose a run the user started.
    return {
      details: [job.kind, "Its outcome is no longer recorded."].join("\n"),
      severity: ENotificationSeverity.INFO,
      source: job.kind,
      title: "Ended while the window was reloading",
    };
  }

  return {
    details: [job.kind, ...(error ? [error] : [])].join("\n"),
    severity: ADOPTED_SEVERITIES[conclusion],
    source: job.kind,
    title: ADOPTED_TITLES[conclusion],
  };
}
