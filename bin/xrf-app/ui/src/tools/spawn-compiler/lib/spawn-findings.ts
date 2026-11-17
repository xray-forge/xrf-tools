import { CheckReport, Finding } from "@xrf/ipc/types/xrf-report";
import { SpawnRootOutcome } from "@xrf/ipc/types/xrf-spawn-builder";

/**
 * @param spawn - What a build wrote.
 * @returns Every finding its report holds, check by check: the graph points left out, the connections to levels not
 *   built, and the restrictors cutting an AI map apart, each naming its level.
 */
export function listSpawnFindings(spawn: SpawnRootOutcome): Array<Finding> {
  return spawn.report.checks.flatMap((check: CheckReport) => check.findings);
}

/**
 * @param spawn - What a build wrote.
 * @returns How many findings its report holds.
 */
export function countSpawnFindings(spawn: SpawnRootOutcome): number {
  return listSpawnFindings(spawn).length;
}
