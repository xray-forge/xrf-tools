import { RenderLoadFailure } from "@/core/ipc/types/xrf-renderer";
import { ILevelSpawnReport } from "@/core/level/lib/spawn/level-spawn-report";
import { IEditorProblem } from "@/core/shell/editor/EditorProblemsPanel";

import { ELevelProblemRule } from "./level-problem-rule";

/** What the game keeps every level's spawn in, which a failure to read it names. */
const SPAWN_FILE: string = "spawns\\all.spawn";

/**
 * The spawn where it could not be listed, which leaves no spawned object drawn, then every visual the spawned objects
 * name that could not be read, so none of the objects standing as it is drawn.
 *
 * @param report - What the spawn's listing came to.
 * @param models - The visuals that could not be read.
 * @returns The rows, the spawn's first, each visual's by its path.
 */
export function listSpawnProblems(
  report: ILevelSpawnReport,
  models: ReadonlyArray<RenderLoadFailure> = []
): Array<IEditorProblem> {
  const spawn: Array<IEditorProblem> = report.failure
    ? [{ message: `No spawned object is drawn: ${report.failure}`, rule: ELevelProblemRule.SPAWN, subject: SPAWN_FILE }]
    : [];

  return [
    ...spawn,
    ...models.map(({ name, reason }: RenderLoadFailure) => ({
      message: `Its objects are not drawn: ${reason}`,
      rule: ELevelProblemRule.SPAWN,
      subject: name,
    })),
  ];
}
