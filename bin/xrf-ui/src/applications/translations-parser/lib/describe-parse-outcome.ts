import { formatCount } from "@xrf/format";

import { TranslationParseResult } from "@/core/ipc/types/xrf-translation";
import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

import { listTranslationParseFindings } from "./parse-findings";

/**
 * What the notification centre says when a translation import ends.
 *
 * A stopped import leaves the sources it had already written complete and the rest untouched, because each is written
 * whole through a staged replace. A dry run says what it would have changed, since it wrote nothing at all.
 *
 * @param output - Directory the JSON sources were written into.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeTranslationParseOutcome(
  output: string,
  outcome: IJobOutcome<TranslationParseResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not import translations", output, error);
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Read ${formatCount(result.census.filesRead)} table(s) before stopping.`,
        "What was written was left in place; running it again finishes the job.",
        output,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped importing translations",
    };
  }

  const findings: number = result ? listTranslationParseFindings(result).length : 0;

  if (result?.isDryRun) {
    return {
      details: output,
      severity: ENotificationSeverity.INFO,
      title: `Import would change ${result.census.filesCreated + result.census.filesUpdated} file(s)`,
    };
  }

  return {
    details: output,
    severity: findings ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: `Imported ${result?.census.filesRead ?? 0} table(s)${findings ? `, ${findings} finding(s)` : ""}`,
  };
}
