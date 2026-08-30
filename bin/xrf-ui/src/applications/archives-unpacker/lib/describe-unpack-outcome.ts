import { ArchiveUnpackResult } from "@/core/bindings/types/xrf-pack";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when an unpack ends.
 *
 * Pure over its arguments, and it has to stay that way: the jobs service calls it when the run settles, which may be
 * long after this application's container was torn down, so anything it reached into could already be gone.
 *
 * A cancelled unpack is informational rather than a warning. What it leaves behind is a real but partial tree, not the
 * unusable volume set a cancelled pack leaves, so it says how far it got and stops there.
 *
 * @param source - Directory the archives were read from.
 * @param destination - Directory they were written into.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeUnpackOutcome(
  source: string,
  destination: string,
  outcome: IJobOutcome<ArchiveUnpackResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [source, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not unpack archives",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Stopped after ${result.filesUnpacked.toLocaleString()} of ${result.filesTotal.toLocaleString()} entries.`,
        "What was written was left in place:",
        destination,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped unpacking archives",
    };
  }

  return {
    details: [source, destination].join("\n"),
    severity: ENotificationSeverity.SUCCESS,
    title: "Unpacked archives",
  };
}
