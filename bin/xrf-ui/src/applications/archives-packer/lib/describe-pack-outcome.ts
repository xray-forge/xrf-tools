import { ArchivePackConfig, ArchivePackResult } from "@/core/bindings/types/xrf-pack";
import { IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { EApplicationId } from "@/core/routing/application";

/**
 * What the notification centre says when a pack ends.
 *
 * @param config - Configuration the run was given.
 * @param outcome - How the run ended.
 * @returns The notification to record.
 */
export function describePackOutcome(
  config: ArchivePackConfig,
  outcome: IJobOutcome<ArchivePackResult>
): INotificationPayload {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [config.source, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      source: EApplicationId.ARCHIVES_PACKER,
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
      source: EApplicationId.ARCHIVES_PACKER,
      title: "Stopped packing archives",
    };
  }

  return {
    details: [config.source, config.destination].join("\n"),
    severity: ENotificationSeverity.SUCCESS,
    source: EApplicationId.ARCHIVES_PACKER,
    title: "Packed archives",
  };
}
