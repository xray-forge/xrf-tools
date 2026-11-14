import { ESceneProblemKind, SceneProblem, SceneProblemKind, SceneToolDescription } from "@xrf/ipc/types/xrf-app-levels";
import { assertExhaustive, Maybe } from "@xrf/types";

import { IEditorProblem } from "@/core/shell/chrome/EditorProblemsPanel";
import { getSceneToolLabel } from "@/tools/level-editor/lib/tools/scene-tools";

/** Everything a scene's problems are listed from: what its open found, and what reading each of its parts found. */
export interface ILevelEditorProblemSources {
  /** Every tool the scene holds part files of, which names the tool a problem is on. */
  tools: ReadonlyArray<SceneToolDescription>;
  /** What the open found of the scene's part files. */
  scene: ReadonlyArray<SceneProblem>;
  /** The object tools whose objects did not read. */
  objects: ReadonlyArray<SceneProblem>;
  /** The binary tools whose parts did not read. */
  binaryTools: ReadonlyArray<SceneProblem>;
}

/**
 * Everything the SDK's Level Editor would skip or refuse of a scene, as rows: what the open found first, then what its
 * object tools' and binary tools' reads found.
 *
 * @param sources - What the problems are listed from.
 * @returns One row a problem, named by its kind, with its tool's label and its part file, where it has one, as the
 *   subject the row shows and the file as where it lies.
 */
export function listLevelEditorProblems(sources: ILevelEditorProblemSources): Array<IEditorProblem> {
  const tools: ReadonlyMap<string, SceneToolDescription> = new Map(
    sources.tools.map((tool: SceneToolDescription) => [tool.name, tool])
  );

  return [...sources.scene, ...sources.objects, ...sources.binaryTools].map((problem: SceneProblem): IEditorProblem => {
    const tool: Maybe<SceneToolDescription> = tools.get(problem.tool);
    const label: string = getSceneToolLabel(tool?.kind ?? null, problem.tool);

    return {
      ...(problem.file ? { location: { line: null, path: problem.file } } : {}),
      message: problem.message,
      rule: describeSceneProblemKind(problem.kind),
      subject: problem.file ? `${label} · ${problem.file}` : label,
    };
  });
}

/**
 * @param kind - Why the editor would skip or refuse a part.
 * @returns It as a problem's rule names it.
 */
export function describeSceneProblemKind(kind: SceneProblemKind): string {
  switch (kind) {
    case ESceneProblemKind.UNREADABLE:
      return "Unreadable";

    case ESceneProblemKind.FOREIGN_GUID:
      return "Another scene's part";

    case ESceneProblemKind.WRONG_VERSION:
      return "Wrong tool version";

    case ESceneProblemKind.FOREIGN_CLASS:
      return "Another tool's class";

    default:
      return assertExhaustive(kind);
  }
}
