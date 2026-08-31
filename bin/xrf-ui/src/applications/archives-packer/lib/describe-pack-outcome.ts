import { ArchivePackConfig, ArchivePackResult } from "@/core/bindings/types/xrf-pack";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a pack ends.
 *
 * @param config - Configuration the run was given.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describePackOutcome(config: ArchivePackConfig, outcome: IJobOutcome<ArchivePackResult>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [config.source, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not pack archives",
    };
  }

  if (result?.outcome === "cancelled") {
    // A stopped run takes back the volumes it made, so ordinarily there is nothing to clean up. A run that was allowed
    // to replace an existing set is the exception: there the same paths may have held a working archive, deleting them
    // would compound the loss, and these are the files somebody has to look at.
    return {
      details: result.volumesOpened.length
        ? [
            `Stopped after ${result.volumesOpened.length} volume(s). These files are incomplete and were not removed:`,
            ...result.volumesOpened,
          ].join("\n")
        : [config.destination, "Nothing was published: the output directory is as it was."].join("\n"),
      severity: ENotificationSeverity.WARNING,
      title: "Stopped packing archives",
    };
  }

  return {
    details: [config.source, config.destination].join("\n"),
    severity: ENotificationSeverity.SUCCESS,
    title: "Packed archives",
  };
}
