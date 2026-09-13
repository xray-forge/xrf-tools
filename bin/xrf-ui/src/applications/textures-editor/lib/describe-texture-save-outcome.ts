import { TextureSaveOutcome } from "@/core/ipc/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a texture save ends.
 *
 * @param reference - Engine reference of the texture that was saved.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeTextureSaveOutcome(reference: string, outcome: IJobOutcome<TextureSaveOutcome>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [reference, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not save texture",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [reference, "Nothing was written: both files are prepared before either is published."].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Saving the texture was stopped",
    };
  }

  return {
    details: [reference, ...(result?.written ?? [])].join("\n"),
    severity: ENotificationSeverity.SUCCESS,
    title: `Saved ${result?.written.length ?? 0} texture file(s)`,
  };
}
