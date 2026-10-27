import { formatCount } from "@xrf/format";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { EquipmentSpritePackResult } from "@xrf/ipc/types/xrf-texture";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when an equipment sprite pack ends.
 *
 * A stopped run says plainly that nothing was written. The sheet is one image encoded and saved at the very end, so
 * stopping before that leaves the destination exactly as it was — unlike an archive pack, which has already replaced
 * files by the time anyone can stop it.
 *
 * @param output - File the sprite sheet would have been written to.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describePackSpriteOutcome(output: string, outcome: IJobOutcome<EquipmentSpritePackResult>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not pack equipment sprite", output, error);
  }

  if (result?.outcome === EJobOutcome.CANCELLED) {
    return {
      details: ["Nothing was written; the sheet is saved only once every icon is drawn.", output].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped packing equipment sprite",
    };
  }

  return {
    details: output,
    severity: result?.skippedCount ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: result?.skippedCount
      ? `Packed ${formatCount(result.packedCount)} icon(s), ${formatCount(result.skippedCount)} without an image`
      : `Packed ${formatCount(result?.packedCount ?? 0)} icon(s)`,
  };
}
