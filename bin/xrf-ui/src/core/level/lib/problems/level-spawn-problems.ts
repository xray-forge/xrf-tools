import { LevelSpawnModelFailure } from "@/core/ipc/types/xrf-app";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

import { ELevelProblemRule } from "./level-problem-rule";

/** What the game keeps every level's spawn in, which a failure to read it names. */
const SPAWN_FILE: string = "spawns\\all.spawn";

/**
 * The spawn's read where it stopped, which leaves every visual not read by then undrawn, then every visual the spawned
 * objects name that could not be read, so none of the objects standing as it is drawn.
 *
 * @param report - What the spawn's read came to.
 * @returns The rows, the spawn's first, each visual's by its path.
 */
export function listSpawnProblems(report: ILevelSpawnReport): Array<IEditorProblem> {
  const spawn: Array<IEditorProblem> = report.failure
    ? [{ message: toStoppedMessage(report, report.failure), rule: ELevelProblemRule.SPAWN, subject: SPAWN_FILE }]
    : [];

  return [
    ...spawn,
    ...report.failures.map(({ name, reason }: LevelSpawnModelFailure) => ({
      message: `Its objects are not drawn: ${reason}`,
      rule: ELevelProblemRule.SPAWN,
      subject: name,
    })),
  ];
}

/** What a read stopped by a failure leaves undrawn: nothing, or the visuals past those read by then. */
function toStoppedMessage(report: ILevelSpawnReport, failure: string): string {
  return report.read === 0
    ? `No spawned object is drawn: ${failure}`
    : `The objects of ${report.visuals - report.read} of ${report.visuals} visuals are not drawn: ${failure}`;
}
