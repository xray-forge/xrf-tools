import { LevelSpawnModelFailure } from "@/core/ipc/types/xrf-app";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

import { ELevelProblemRule } from "./level-problem-rule";

/** What the game keeps every level's spawn in, which a failure to read it names. */
const SPAWN_FILE: string = "spawns\\all.spawn";

/**
 * The spawn itself where it could not be read, which leaves the level without a spawned object, then every visual the
 * spawned objects name that could not be read, so none of the objects standing as it is drawn.
 *
 * @param report - What the spawn's read came to.
 * @returns The rows, the spawn's first, each visual's by its path.
 */
export function listSpawnProblems(report: ILevelSpawnReport): Array<IEditorProblem> {
  const spawn: Array<IEditorProblem> = report.failure
    ? [{ message: `No spawned object is drawn: ${report.failure}`, rule: ELevelProblemRule.SPAWN, subject: SPAWN_FILE }]
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
