import { formatCount } from "@xrf/format";
import { SpawnBuildResult } from "@xrf/ipc/types/xrf-app-spawn";
import { SpawnRootOutcome, SpawnRootRequest } from "@xrf/ipc/types/xrf-spawn-builder";
import { Nullable } from "@xrf/types";

import { describeJobFailure, IJobNotice, IJobOutcome } from "@/core/jobs/lib";
import { ENotificationSeverity } from "@/core/notifications/lib";
import { countSpawnFindings } from "@/tools/spawn-compiler/lib/spawn-findings";

/**
 * What the notification centre says when a spawn's build ends.
 *
 * Pure over its arguments: the jobs service calls it when the run settles, possibly after this tool was closed.
 *
 * A finished build that reported findings is a warning one: a connection left out or a restrictor cutting an AI map
 * apart is written as the game would get it, and is worth reading before the spawn is played.
 *
 * @param request - What the run was asked to do.
 * @param outcome - How the run ended.
 * @returns What to record about it.
 */
export function describeSpawnBuildOutcome(
  request: SpawnRootRequest,
  outcome: IJobOutcome<SpawnBuildResult>
): IJobNotice {
  const { result, error } = outcome;

  if (error) {
    return describeJobFailure("Could not build the spawn", request.name, error);
  }

  const spawn: Nullable<SpawnRootOutcome> = result?.spawn ?? null;

  if (!spawn) {
    return {
      details: ["Stopped before writing; nothing was written.", request.destination ?? request.gamedata].join("\n"),
      severity: ENotificationSeverity.INFO,
      title: "Stopped building the spawn",
    };
  }

  const findings: number = countSpawnFindings(spawn);

  return {
    details: [
      spawn.path,
      `${formatCount(spawn.levels.length)} levels: ${spawn.levels.join(", ")}.`,
      `${formatCount(spawn.objects)} objects, ${formatCount(spawn.artefactPoints)} artefact points, ` +
        `${formatCount(spawn.patrols)} patrols.`,
      ...(findings > 0 ? [`${formatCount(findings)} findings reported.`] : []),
    ].join("\n"),
    severity: findings > 0 ? ENotificationSeverity.WARNING : ENotificationSeverity.SUCCESS,
    title: "Built the spawn",
  };
}
