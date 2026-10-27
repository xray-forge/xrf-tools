import { formatCount } from "@xrf/format";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { LtxProjectFormatResult } from "@xrf/ipc/types/xrf-ltx";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a configs formatting run ends.
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
    return describeJobFailure(isCheck ? "Could not check formatting" : "Could not format configs", directory, error);
  }

  if (result?.outcome === EJobOutcome.CANCELLED) {
    return {
      details: [
        isCheck
          ? `Read ${formatCount(result.totalFiles)} file(s), ${formatCount(result.invalidFiles)} badly formatted so far.`
          : `Read ${formatCount(result.totalFiles)} file(s), of which ${formatCount(result.invalidFiles)} were rewritten.`,
        isCheck
          ? "The rest of the project was not read."
          : "The rest were left as they were; running it again finishes the job.",
        directory,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: isCheck ? "Stopped checking formatting" : "Stopped formatting configs",
    };
  }

  const invalid: number = result?.invalidFiles ?? 0;
  const total: number = result?.totalFiles ?? 0;

  if (isCheck) {
    return {
      details: directory,
      severity: invalid ? ENotificationSeverity.ERROR : ENotificationSeverity.SUCCESS,
      title: invalid
        ? `${formatCount(invalid)} of ${formatCount(total)} config file(s) have invalid formatting`
        : "All config files are in correct format",
    };
  }

  return {
    details: directory,
    severity: ENotificationSeverity.SUCCESS,
    title: `Formatted ${formatCount(invalid)} of ${formatCount(total)} config file(s)`,
  };
}
