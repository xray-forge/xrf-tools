import { ProjectFormatResult } from "@/core/bindings/types/xrf-translation";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a translations formatting run ends.
 *
 * @param directory - Translations directory the run was pointed at.
 * @param isCheck - Whether the run reported the formatting rather than repairing it.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeFormatOutcome(
  directory: string,
  isCheck: boolean,
  outcome: IJobOutcome<ProjectFormatResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [directory, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: isCheck ? "Could not check translation formatting" : "Could not format translation sources",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        isCheck
          ? `Read ${result.totalFiles.toLocaleString()} source(s), ${result.invalidFiles} unformatted so far.`
          : `Read ${result.totalFiles.toLocaleString()} source(s), of which ${result.invalidFiles} were rewritten.`,
        isCheck
          ? "The rest of the tree was not read."
          : "The rest were left as they were; running it again finishes the job.",
        directory,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: isCheck ? "Stopped checking translation formatting" : "Stopped formatting translation sources",
    };
  }

  const invalid: number = result?.invalidFiles ?? 0;
  const total: number = result?.totalFiles ?? 0;

  if (isCheck) {
    return {
      details: directory,
      severity: invalid ? ENotificationSeverity.ERROR : ENotificationSeverity.SUCCESS,
      title: invalid
        ? `${invalid.toLocaleString()} of ${total.toLocaleString()} translation source(s) are not formatted`
        : "All translation sources are in correct format",
    };
  }

  return {
    details: directory,
    severity: ENotificationSeverity.SUCCESS,
    title: `Formatted ${invalid.toLocaleString()} of ${total.toLocaleString()} translation source(s)`,
  };
}
