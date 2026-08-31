import { TranslationVerifySummary } from "@/core/bindings/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
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
  outcome: IJobOutcome<TranslationVerifySummary>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [sources, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not check translations",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [`Checked ${result.checked.toLocaleString()} id(s) before stopping.`, sources].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped checking translations",
    };
  }

  return {
    details: sources,
    severity: result?.missing ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: result?.missing ? `Translations are incomplete: ${result.missing} missing` : "Translations are complete",
  };
}
