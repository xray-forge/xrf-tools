import { describe, expect, it } from "@jest/globals";

import { describeAdoptedOutcome } from "@/core/jobs/lib/describe-adopted-outcome";
import { IJobState } from "@/core/jobs/lib/jobs-types";
import { ENotificationSeverity } from "@/core/notifications/lib";

const JOB: IJobState = {
  id: "b8f0",
  kind: "archives.pack",
  progress: null,
  request: null,
  isCancelRequested: false,
  isAdopted: true,
};

describe("describeAdoptedOutcome", () => {
  it("reports a completed run as a success", () => {
    expect(describeAdoptedOutcome(JOB, "completed", null).severity).toBe(ENotificationSeverity.SUCCESS);
  });

  it("reports a cancelled run as information", () => {
    expect(describeAdoptedOutcome(JOB, "cancelled", null).severity).toBe(ENotificationSeverity.INFO);
  });

  it("reports a failed run with the reason the backend recorded", () => {
    const notification = describeAdoptedOutcome(JOB, "failed", "volume cap refuses particles.xr");

    expect(notification.severity).toBe(ENotificationSeverity.ERROR);
    expect(notification.details).toContain("volume cap refuses particles.xr");
  });

  it("says the outcome is unknown rather than guessing at one", () => {
    // A job that finished and then fell out of the retained listing, which takes twenty finished jobs. Reporting it as
    // completed would be a guess; saying nothing would lose a run the user started.
    const notification = describeAdoptedOutcome(JOB, null, null);

    expect(notification.severity).toBe(ENotificationSeverity.INFO);
    expect(notification.details).toContain("no longer recorded");
  });

  it("names the kind, since an adopted job has no tool to attribute it to", () => {
    // The command answered a page that no longer exists, so the descriptor naming the tool went with it.
    expect(describeAdoptedOutcome(JOB, "completed", null).source).toBe("archives.pack");
  });
});
