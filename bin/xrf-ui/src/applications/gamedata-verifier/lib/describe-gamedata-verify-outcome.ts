import { GamedataCheckSummary, GamedataVerifySummary } from "@/core/bindings/types/xrf-app";
import { IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * @param summary - Verdict to count.
 * @returns Checks that did not pass and were not skipped.
 */
export function selectFailedChecks(summary: GamedataVerifySummary): Array<GamedataCheckSummary> {
  return summary.checks.filter(
    (check: GamedataCheckSummary) => check.status !== "passed" && check.status !== "skipped"
  );
}

/**
 * What the notification centre says when a gamedata verification ends.
 *
 * A stopped run is information rather than a verdict. The checks that ran are real answers, but the ones that never
 * started said nothing — and reporting that silence as a pass is the one way a partial check can do harm.
 *
 * `incomplete` is kept distinct from `failed`: a check that could not read everything it needed has not found a
 * problem, it has found that it could not look.
 *
 * @param root - Gamedata root the run was pointed at.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeGamedataVerifyOutcome(root: string, outcome: IJobOutcome<GamedataVerifySummary>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return {
      details: [root, error.message].join("\n"),
      severity: ENotificationSeverity.ERROR,
      title: "Could not verify gamedata",
    };
  }

  if (result?.outcome === "cancelled") {
    return {
      details: [
        `Ran ${result.checks.length} check(s) before stopping.`,
        "The rest were not run, so nothing is known about them.",
        root,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped verifying gamedata",
    };
  }

  if (!result) {
    return { details: root, severity: ENotificationSeverity.INFO, title: "Verified gamedata" };
  }

  const failed: Array<GamedataCheckSummary> = selectFailedChecks(result);

  if (!failed.length) {
    return {
      details: root,
      severity: ENotificationSeverity.SUCCESS,
      title: `Gamedata passed ${result.checks.length} check(s)`,
    };
  }

  return {
    details: [root, ...failed.map((check: GamedataCheckSummary) => `${check.check}: ${check.summary}`)].join("\n"),
    severity: result.status === "incomplete" ? ENotificationSeverity.WARNING : ENotificationSeverity.ERROR,
    title: `Gamedata ${result.status}: ${failed.length} of ${result.checks.length} check(s)`,
  };
}
