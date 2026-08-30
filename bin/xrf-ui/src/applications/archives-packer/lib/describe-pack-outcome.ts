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
export function describePackOutcome(
  config: ArchivePackConfig,
  outcome: IJobOutcome<ArchivePackResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [config.source, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not pack archives",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Stopped after ${result.volumesOpened.length} volume(s). These files are incomplete and were not removed:`,
        ...result.volumesOpened,
      ].join("\n"),
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
