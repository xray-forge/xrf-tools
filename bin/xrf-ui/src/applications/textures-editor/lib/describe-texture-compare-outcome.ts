import { TextureEncodingComparison } from "@/core/ipc/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a format comparison ends.
 *
 * @param reference - Engine reference of the texture that was weighed.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeTextureCompareOutcome(
  reference: string,
  outcome: IJobOutcome<TextureEncodingComparison>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [reference, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not compare texture formats",
    };
  }

  const count: number = result?.candidates.length ?? 0;

  if (result?.outcome === "cancelled") {
    return {
      details: [reference, `Weighed ${count} format(s) before stopping; those are still available to save.`].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Comparing the texture formats was stopped",
    };
  }

  return {
    details: [reference, ...(result?.candidates ?? []).map((candidate) => candidate.label)].join("\n"),
    severity: ENotificationSeverity.SUCCESS,
    title: `Weighed ${count} texture format(s)`,
  };
}
