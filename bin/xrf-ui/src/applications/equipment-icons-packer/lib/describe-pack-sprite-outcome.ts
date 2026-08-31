import { PackEquipmentResult } from "@/core/bindings/types/xrf-texture";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
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
export function describePackSpriteOutcome(output: string, outcome: IJobOutcome<PackEquipmentResult>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [output, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not pack equipment sprite",
    };
  }

  if (result?.outcome === "cancelled") {
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
      ? `Packed ${result.packedCount} icon(s), ${result.skippedCount} without an image`
      : `Packed ${result?.packedCount ?? 0} icon(s)`,
  };
}
