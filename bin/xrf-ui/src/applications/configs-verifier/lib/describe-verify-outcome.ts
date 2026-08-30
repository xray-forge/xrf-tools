import { LtxProjectVerifyResult } from "@/core/bindings/types/xrf-ltx";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a configs verification ends.
 *
 * A stopped run is reported as information rather than as a verdict. It read part of the project, so its findings are
 * real but its silence is not: calling that "passed" would be the one way a partial check can mislead.
 *
 * @param directory - Configs directory the run was pointed at.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeVerifyOutcome(
  directory: string,
  outcome: IJobOutcome<LtxProjectVerifyResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [directory, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not verify configs",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Stopped after ${result.totalFiles.toLocaleString()} file(s).`,
        `${result.errors.length} problem(s) found so far.`,
        "The rest of the project was not read.",
        directory,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped verifying configs",
    };
  }

  if (result?.errors.length) {
    return {
      details: directory,
      severity: ENotificationSeverity.WARNING,
      title: `Configs did not pass validation: ${result.errors.length} problem(s)`,
    };
  }

  return {
    details: directory,
    severity: ENotificationSeverity.SUCCESS,
    title: "Configs passed validation",
  };
}
