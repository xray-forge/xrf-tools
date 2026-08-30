import { describe, expect, it } from "@jest/globals";

import { describeUnpackOutcome } from "@/applications/archives-unpacker/lib/describe-unpack-outcome";
import { ArchiveUnpackResult } from "@/core/bindings/types/xrf-pack";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

const SOURCE: string = "C:\\Games\\Anomaly\\db";
const DESTINATION: string = "C:\\work\\unpacked";

function result(patch: Partial<ArchiveUnpackResult> = {}): ArchiveUnpackResult {
  return { outcome: "completed", filesTotal: 1000, filesUnpacked: 1000, ...patch } as ArchiveUnpackResult;
}

function outcome(patch: Partial<IJobOutcome<ArchiveUnpackResult>>): IJobOutcome<ArchiveUnpackResult> {
  return { isCancelRequested: false, result: null, error: null, ...patch };
}

describe("describeUnpackOutcome", () => {
  it("reports a finished unpack as a success naming both ends", () => {
    const finished: IJobOutcome<ArchiveUnpackResult> = outcome({ result: result() });
    const notification: IJobNotice = describeUnpackOutcome(SOURCE, DESTINATION, finished);

    expect(notification.severity).toBe(ENotificationSeverity.SUCCESS);
    expect(notification.details).toContain(SOURCE);
    expect(notification.details).toContain(DESTINATION);
  });

  it("reports a failure with the reason", () => {
    const notification: IJobNotice = describeUnpackOutcome(
      SOURCE,
      DESTINATION,
      outcome({ error: new Error("archive is not readable") })
    );

    expect(notification.severity).toBe(ENotificationSeverity.ERROR);
    expect(notification.details).toContain("archive is not readable");
  });

  it("reports a cancelled unpack as information, with how far it got", () => {
    // Softer than the pack case on purpose: what is left is a real but partial tree of the user's own files, not an
    // unusable volume set, so it says where it stopped rather than warning about wreckage.
    const notification: IJobNotice = describeUnpackOutcome(
      SOURCE,
      DESTINATION,
      outcome({
        isCancelRequested: true,
        result: result({ outcome: "cancelled", filesTotal: 100000, filesUnpacked: 45000 }),
      })
    );

    expect(notification.severity).toBe(ENotificationSeverity.INFO);
    expect(notification.details).toContain((45000).toLocaleString());
    expect(notification.details).toContain((100000).toLocaleString());
    expect(notification.details).toContain(DESTINATION);
  });

  it("reads the outcome off the result rather than off the cancel request", () => {
    // A run can finish ahead of the request to stop it, and reporting that as stopped would understate what landed.
    const notification: IJobNotice = describeUnpackOutcome(
      SOURCE,
      DESTINATION,
      outcome({ isCancelRequested: true, result: result() })
    );

    expect(notification.severity).toBe(ENotificationSeverity.SUCCESS);
  });
});
