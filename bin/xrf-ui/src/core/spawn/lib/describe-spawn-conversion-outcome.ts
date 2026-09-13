import { SpawnConversion, SpawnConversionResult } from "@/core/ipc/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * Describes the backend's actual conversion outcome, including a cancellation accepted before writing.
 *
 * @param operation - Requested conversion.
 * @param source - Input path.
 * @param destination - Output path.
 * @param outcome - Result or normalized failure from the jobs service.
 * @returns Notification independent of the lifetime of the initiating form.
 */
export function describeSpawnConversionOutcome(
  operation: SpawnConversion,
  source: string,
  destination: string,
  outcome: IJobOutcome<SpawnConversionResult>
): IJobNotice {
  if (outcome.error) {
    return {
      title: `Could not ${operation} spawn file`,
      severity: ENotificationSeverity.ERROR,
      details: [source, outcome.error.message].join("\n"),
    };
  }

  if (outcome.result?.outcome === "cancelled") {
    return {
      title: `Stopped ${operation === "pack" ? "packing" : "unpacking"} spawn file`,
      severity: ENotificationSeverity.INFO,
      details: "Stopped before writing. Output was left unchanged.",
    };
  }

  return {
    title: `${operation === "pack" ? "Packed" : "Unpacked"} spawn file`,
    severity: ENotificationSeverity.SUCCESS,
    details: [source, destination].join("\n"),
  };
}
