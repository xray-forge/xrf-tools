import { formatCount } from "@xrf/format";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { TranslationVerifyResult } from "@xrf/ipc/types/xrf-translation";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a translation check ends.
 *
 * A stopped run is information rather than a verdict: it read part of the tree, so its silence about the rest says
 * nothing, and reporting it as complete would be the one way a partial check can mislead. What the game would refuse
 * or show wrong outranks a missing translation, which the game shows as its id.
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

  if (result?.outcome === EJobOutcome.CANCELLED) {
    return {
      details: [`Checked ${formatCount(result.checkedTranslationsCount)} id(s) before stopping.`, sources].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped checking translations",
    };
  }

  const problems: number = result?.problems.length ?? 0;

  if (problems) {
    return {
      details: sources,
      severity: ENotificationSeverity.ERROR,
      title: `Translations the game would refuse or show wrong: ${problems} problem(s)`,
    };
  }

  const missing: number = result?.missingTranslationsCount ?? 0;

  return {
    details: sources,
    severity: missing ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: missing ? `Translations are incomplete: ${missing} missing` : "Translations are complete",
  };
}
