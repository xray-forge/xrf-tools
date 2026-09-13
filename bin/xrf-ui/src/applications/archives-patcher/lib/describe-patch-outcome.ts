import { ArchivePatchConfig, ArchivePatchResult } from "@/core/ipc/types/xrf-pack";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * Builds a notification for a finished comparison or patch.
 *
 * Must remain independent of application state: jobs may finish after the view is disposed.
 *
 * @param config - Run configuration.
 * @param outcome - Result or error.
 * @returns The job notification.
 */
export function describePatchOutcome(config: ArchivePatchConfig, outcome: IJobOutcome<ArchivePatchResult>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [config.target ?? config.input, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not build patch",
    };
  }

  if (!result) {
    return { severity: ENotificationSeverity.ERROR, title: "Could not build patch" };
  }

  const carried: number = result.added.length + result.modified.length;

  if (result.outcome === "cancelled") {
    return {
      details: "Nothing was published; the destination was left as it was found.",
      severity: ENotificationSeverity.INFO,
      title: "Stopped building patch",
    };
  }

  switch (result.publication.kind) {
    case "published":
      return {
        details: [
          `${carried.toLocaleString()} entry(s) carried into ${result.publication.volumes.length} volume(s).`,
          ...result.publication.volumes,
        ].join("\n"),
        severity: ENotificationSeverity.SUCCESS,
        title: "Published patch",
      };

    case "unnecessary":
      return {
        details: "The two worlds hold the same files, so no patch was written.",
        severity: ENotificationSeverity.INFO,
        title: "Nothing to patch",
      };

    default:
      return {
        details: `${carried.toLocaleString()} entry(s) would be carried.`,
        severity: ENotificationSeverity.INFO,
        title: "Compared archives",
      };
  }
}
