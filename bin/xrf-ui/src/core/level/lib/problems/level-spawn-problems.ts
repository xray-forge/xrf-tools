import { LevelSpawnModelFailure } from "@/core/ipc/types/xrf-app";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

import { ELevelProblemRule } from "./level-problem-rule";

/**
 * Every visual the spawned objects name that could not be read, so none of the objects standing as it is drawn.
 *
 * @param failures - The visuals, and why each could not be read.
 * @returns The rows, each naming the visual by its path.
 */
export function listSpawnProblems(failures: ReadonlyArray<LevelSpawnModelFailure>): Array<IEditorProblem> {
  return failures.map(({ name, reason }: LevelSpawnModelFailure) => ({
    message: `Its objects are not drawn: ${reason}`,
    rule: ELevelProblemRule.SPAWN,
    subject: name,
  }));
}
