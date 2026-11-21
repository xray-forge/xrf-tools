import { TextureSaveAsOutcome } from "@xrf/ipc/types/xrf-app-textures";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a Save as ends.
 *
 * @param reference - Texture name of the original.
 * @param destination - Texture name asked for the copy.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeTextureSaveAsOutcome(
  reference: string,
  destination: string,
  outcome: IJobOutcome<TextureSaveAsOutcome>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not save texture as", `${reference} as ${destination}`, error);
  }

  if (result?.outcome === EJobOutcome.CANCELLED) {
    return {
      details: [destination, "Nothing was written: the copy's files are written together or not at all."].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Save as was stopped",
    };
  }

  return {
    details: [result?.reference ?? destination, ...(result?.written ?? [])].join("\n"),
    severity: ENotificationSeverity.SUCCESS,
    title: `Saved ${reference} as ${result?.reference ?? destination}`,
  };
}
