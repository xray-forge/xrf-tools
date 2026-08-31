import { ArchiveExtractDirectoryResult } from "@/core/bindings/types/xrf-pack";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a directory extraction ends.
 *
 * A stopped run is information rather than a warning. What it leaves behind is a real but partial tree of the archive's
 * own files, and nothing removes them — the same shape a stopped unpack has, because it is the same writing.
 *
 * @param prefix - Archive-relative directory the run was pointed at; empty selects the archive root.
 * @param destination - Directory the entries were written into.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeExtractOutcome(
  prefix: string,
  destination: string,
  outcome: IJobOutcome<ArchiveExtractDirectoryResult>
): IJobNotice {
  const { result, error } = outcome;
  const from: string = prefix || "the archive root";

  if (error) {
    return {
      details: [destination, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: `Could not extract ${from}`,
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Stopped after ${result.extractedCount.toLocaleString()} file(s).`,
        "What was written was left in place:",
        destination,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: `Stopped extracting ${from}`,
    };
  }

  return {
    details: destination,
    severity: ENotificationSeverity.SUCCESS,
    // Reported without a count rather than not at all: a response the parser did not fill in is no reason to turn a
    // write that happened into silence.
    title: result ? `Extracted ${result.extractedCount.toLocaleString()} file(s) from ${from}` : `Extracted ${from}`,
  };
}
