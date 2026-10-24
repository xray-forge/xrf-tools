import { formatCount } from "@xrf/format";

import { TranslationVerifyResult } from "@/core/ipc/types/xrf-translation";
import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a translation check ends.
 *
 * A stopped run is information rather than a verdict: it read part of the tree, so its silence about the rest says
 * nothing, and reporting it as complete would be the one way a partial check can mislead.
 *
 * @param sources - Source tree the run was pointed at.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeTranslationVerifyOutcome(
  sources: string,
  outcome: IJobOutcome<TranslationVerifyResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not check translations", sources, error);
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [`Checked ${formatCount(result.checkedTranslationsCount)} id(s) before stopping.`, sources].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped checking translations",
    };
  }

  const missing: number = result?.missingTranslationsCount ?? 0;

  return {
    details: sources,
    severity: missing ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: missing ? `Translations are incomplete: ${missing} missing` : "Translations are complete",
  };
}
