import { formatCount } from "@xrf/format";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { LtxProjectVerifyResult } from "@xrf/ipc/types/xrf-ltx";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a configs verification ends.
 *
 * @param directory - Configs directory the run was pointed at.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeVerifyOutcome(directory: string, outcome: IJobOutcome<LtxProjectVerifyResult>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not verify configs", directory, error);
  }

  if (result?.outcome === EJobOutcome.CANCELLED) {
    return {
      details: [
        `Stopped after ${formatCount(result.totalFiles)} file(s).`,
        `${formatCount(result.findings.length)} problem(s) found so far.`,
        "The rest of the project was not read.",
        directory,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped verifying configs",
    };
  }

  if (result?.findings.length) {
    return {
      details: directory,
      severity: ENotificationSeverity.WARNING,
      title: `Configs did not pass validation: ${formatCount(result.findings.length)} problem(s)`,
    };
  }

  return {
    details: directory,
    severity: ENotificationSeverity.SUCCESS,
    title: "Configs passed validation",
  };
}
