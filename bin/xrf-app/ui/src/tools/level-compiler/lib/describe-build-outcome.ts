import { formatCount } from "@xrf/format";
import { LevelBuildResult } from "@xrf/ipc/types/xrf-app-levels";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { LevelBuildRequest } from "@xrf/ipc/types/xrf-rawdata";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a build ends.
 *
 * Pure over its arguments: the jobs service calls it when the run settles, possibly after this tool was closed.
 *
 * A finished build that refused a part is a warning one: the level lacks that breakable or ladder, though everything
 * else was written.
 *
 * @param request - What the run was asked to do.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeBuildOutcome(request: LevelBuildRequest, outcome: IJobOutcome<LevelBuildResult>): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not build the level", request.source, error);
  }

  if (!result?.report) {
    return {
      details: ["Stopped before writing; nothing was written.", request.destination].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped building the level",
    };
  }

  if (result.outcome === EJobOutcome.CANCELLED) {
    return {
      details: [
        "The build was written; the compile was stopped, and what it wrote so far stays.",
        request.destination,
      ].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped compiling the level",
    };
  }

  const refused: number = result.report.refused.length;
  const compiled: number = result.compile?.steps.length ?? 0;

  return {
    details: [
      request.source,
      request.destination,
      `${formatCount(result.report.files.length)} files written.`,
      ...(compiled > 0 ? [`${formatCount(compiled)} compile steps run.`] : []),
      ...(refused > 0 ? [`${formatCount(refused)} parts were refused.`] : []),
    ].join("\n"),
    severity: refused > 0 ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: compiled > 0 ? "Built and compiled the level" : "Built the level",
  };
}
