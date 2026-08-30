import { LtxProjectFormatResult } from "@/core/bindings/types/xrf-ltx";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a configs formatting run ends.
 *
 * A stopped run is information rather than a warning. Each file is rewritten through a staged replace, so what it
 * leaves behind is some files formatted and the rest untouched - a state running it again resolves, and never a file
 * half-written.
 *
 * @param directory - Configs directory the run was pointed at.
 * @param isCheck - Whether the run reported the formatting rather than repairing it.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeFormatOutcome(
  directory: string,
  isCheck: boolean,
  outcome: IJobOutcome<LtxProjectFormatResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [directory, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: isCheck ? "Could not check formatting" : "Could not format configs",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Stopped after ${result.totalFiles.toLocaleString()} file(s), of which ${result.invalidFiles} were rewritten.`,
        "The rest were left as they were; running it again finishes the job.",
        directory,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped formatting configs",
    };
  }

  return {
    details: directory,
    severity: ENotificationSeverity.SUCCESS,
    title: `Formatted ${result?.invalidFiles ?? 0} of ${result?.totalFiles ?? 0} config file(s)`,
  };
}
