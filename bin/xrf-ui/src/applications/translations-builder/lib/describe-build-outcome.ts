import { TranslationBuildSummary } from "@/core/bindings/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a translation build ends.
 *
 * A stopped build leaves the string tables it had already written valid and the rest simply absent, so it is reported
 * as information: running it again resolves the difference, and nothing has to be cleaned up.
 *
 * @param output - Directory the string tables were written into.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeTranslationBuildOutcome(
  output: string,
  outcome: IJobOutcome<TranslationBuildSummary>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [output, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not build translations",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Wrote ${result.files.toLocaleString()} string table(s) from ${result.sources.toLocaleString()} source(s).`,
        "The rest were not compiled; running it again finishes the job.",
        output,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped building translations",
    };
  }

  return {
    details: output,
    severity: ENotificationSeverity.SUCCESS,
    title: `Built ${result?.files ?? 0} string table(s) from ${result?.sources ?? 0} source(s)`,
  };
}
