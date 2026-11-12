import { formatCount } from "@xrf/format";
import { LevelDecompileResult } from "@xrf/ipc/types/xrf-app-levels";
import { EJobOutcome } from "@xrf/ipc/types/xrf-job";
import { LevelDecompileRequest } from "@xrf/ipc/types/xrf-rawdata";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";

/**
 * What the notification centre says when a decompile ends.
 *
 * Pure over its arguments: the jobs service calls it when the run settles, possibly after this tool was closed.
 *
 * A finished decompile that left something out is still a success, but a warning one, since the scene lacks what the
 * level held; the result lists each piece with why.
 *
 * @param request - What the run was asked to do.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeDecompileOutcome(
  request: LevelDecompileRequest,
  outcome: IJobOutcome<LevelDecompileResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not decompile the level", request.source, error);
  }

  if (result?.outcome === EJobOutcome.CANCELLED || !result?.report) {
    return {
      details: ["Stopped before writing; nothing was written.", request.source].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped decompiling the level",
    };
  }

  const refused: number = result.report.refused.length;

  return {
    details: [
      request.source,
      request.destination,
      ...(refused > 0 ? [`${formatCount(refused)} pieces the scene cannot hold were left out.`] : []),
    ].join("\n"),
    severity: refused > 0 ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: "Decompiled the level",
  };
}
