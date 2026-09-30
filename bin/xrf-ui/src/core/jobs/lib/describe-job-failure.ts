import { ENotificationSeverity } from "@/core/notifications/lib";

import { IJobNotice } from "./jobs-types";

/**
 * What the notification centre says when a job fails: what it was about, then why.
 *
 * @param title - What could not be done, as the notice's headline.
 * @param subject - The path or reference the job was pointed at.
 * @param error - Why it failed.
 * @returns The error notice.
 */
export function describeJobFailure(title: string, subject: string, error: Error): IJobNotice {
  return {
    details: [subject, error.message].join("\n"),
    severity: ENotificationSeverity.ERROR,
    title,
  };
}
