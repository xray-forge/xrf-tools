import {
  ESceneDrawProblemKind,
  ESceneObjectProperties,
  ESceneProblemKind,
  ESceneToolKind,
  SceneDrawProblem,
  SceneDrawProblemKind,
  SceneObjectDescription,
  SceneObjectId,
  SceneObjectToolDescription,
  SceneProblem,
  SceneProblemKind,
  SceneToolDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { RenderLoadFailure } from "@xrf/ipc/types/xrf-renderer";
import { assertExhaustive, Maybe } from "@xrf/types";

import { listTextureProblems } from "@/core/level/lib/problems/level-texture-problems";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-report";
import { IEditorProblem } from "@/core/shell/chrome/EditorProblemsPanel";
import { getSceneToolLabel } from "@/tools/level-editor/lib/tools/scene-tools";

/** What a library object the renderer could not make a model of is listed under. */
export const LEVEL_EDITOR_MODEL_PROBLEM_RULE: string = "Library object not drawn";

/**
 * Everything a scene's problems are listed from: what its open found, what reading each of its parts found, and what
 * the renderer said of the scene once resident.
 */
export interface ILevelEditorProblemSources {
  /** Every tool the scene holds part files of, which names the tool a problem is on. */
  tools: ReadonlyArray<SceneToolDescription>;
  /** What the open found of the scene's part files. */
  scene: ReadonlyArray<SceneProblem>;
  /** The object tools whose objects did not read. */
  objects: ReadonlyArray<SceneProblem>;
  /** The binary tools whose parts did not read. */
  binaryTools: ReadonlyArray<SceneProblem>;
  /** The scene objects the scene's picture does not draw. */
  drawing: ReadonlyArray<SceneDrawProblem>;
  /** Every object tool's objects, which name the scene object a drawing problem is on and those placing a model. */
  objectTools: ReadonlyArray<SceneObjectToolDescription>;
  /** What the renderer's texture set could not answer for properly. */
  textures: ReadonlyArray<ILevelTextureProblem>;
  /** The library objects the renderer could not make a model of, each named by its reference. */
  models: ReadonlyArray<RenderLoadFailure>;
}

/**
 * Everything the SDK's Level Editor would skip or refuse of a scene, and what of it is not drawn, as rows: what the
 * open found first, then what its object tools' and binary tools' reads found, then the scene objects its picture
 * leaves out, then the textures and the library objects the renderer could not draw.
 *
 * @param sources - What the problems are listed from.
 * @returns One row a problem, named by its kind. A part's problem has its tool's label and its part file, where it has
 *   one, as the subject the row shows and the file as where it lies; a drawing problem has the scene object's name and
 *   the library object it places; a texture problem its reference, as the level viewer lists it; a model problem the
 *   first scene object placing the library object and the library object, the others placing it counted.
 */
export function listLevelEditorProblems(sources: ILevelEditorProblemSources): Array<IEditorProblem> {
  const tools: ReadonlyMap<string, SceneToolDescription> = new Map(
    sources.tools.map((tool: SceneToolDescription) => [tool.name, tool])
  );

  const parts: Array<IEditorProblem> = [...sources.scene, ...sources.objects, ...sources.binaryTools].map(
    (problem: SceneProblem): IEditorProblem => {
      const tool: Maybe<SceneToolDescription> = tools.get(problem.tool);
      const label: string = getSceneToolLabel(tool?.kind ?? null, problem.tool);

      return {
        ...(problem.file ? { location: { line: null, path: problem.file } } : {}),
        message: problem.message,
        rule: describeSceneProblemKind(problem.kind),
        subject: problem.file ? `${label} · ${problem.file}` : label,
      };
    }
  );

  const drawing: Array<IEditorProblem> = sources.drawing.map((problem: SceneDrawProblem): IEditorProblem => ({
    message: problem.message,
    rule: describeSceneDrawProblemKind(problem.kind),
    subject: `${nameSceneObject(sources.objectTools, problem.object)} · ${problem.reference}`,
  }));

  return [
    ...parts,
    ...drawing,
    ...listTextureProblems(sources.textures),
    ...listModelProblems(sources.models, sources.drawing, sources.objectTools),
  ];
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

/**
 * @param kind - Why a scene object is not drawn.
 * @returns It as a problem's rule names it.
 */
export function describeSceneDrawProblemKind(kind: SceneDrawProblemKind): string {
  switch (kind) {
    case ESceneDrawProblemKind.MISSING:
      return "Library object missing";

    case ESceneDrawProblemKind.UNREADABLE:
      return "Library object unreadable";

    default:
      return assertExhaustive(kind);
  }
}

/**
 * Each library object the renderer could not make a model of, with the scene objects standing as it, which are not
 * drawn; one whose `.object` the scene's drawing found unreadable is listed there already, by scene object.
 *
 * @param models - What the renderer could not read or compile, by reference.
 * @param drawing - The scene objects the drawing found not drawn.
 * @param objectTools - Every object tool's objects.
 * @returns One row a library object, its first placing scene object named and the rest counted.
 */
function listModelProblems(
  models: ReadonlyArray<RenderLoadFailure>,
  drawing: ReadonlyArray<SceneDrawProblem>,
  objectTools: ReadonlyArray<SceneObjectToolDescription>
): Array<IEditorProblem> {
  const listed: ReadonlySet<string> = new Set(drawing.map((problem: SceneDrawProblem) => problem.reference));
  const failed: ReadonlyArray<RenderLoadFailure> = models.filter(
    (failure: RenderLoadFailure) => !listed.has(failure.name)
  );

  if (!failed.length) {
    return [];
  }

  const placing: Map<string, Array<string>> = new Map(failed.map((failure: RenderLoadFailure) => [failure.name, []]));
  const objects: ReadonlyArray<SceneObjectDescription> =
    objectTools.find((tool: SceneObjectToolDescription) => tool.tool === ESceneToolKind.SCENE_OBJECT)?.objects ?? [];

  for (const object of objects) {
    if (object.properties.kind === ESceneObjectProperties.SCENE_OBJECT) {
      placing.get(object.properties.reference)?.push(object.name);
    }
  }

  return failed.map(({ name, reason }: RenderLoadFailure) => {
    const [first, ...rest] = placing.get(name) ?? [];

    return {
      message: `${describePlacing(first, rest.length)} not drawn: ${reason}`,
      rule: LEVEL_EDITOR_MODEL_PROBLEM_RULE,
      subject: first ? `${first} · ${name}` : name,
    };
  });
}

/**
 * @param first - The first scene object placing a library object, if the listing holds any.
 * @param others - How many more place it.
 * @returns Who is not drawn, as a sentence's subject.
 */
function describePlacing(first: Maybe<string>, others: number): string {
  if (!first) {
    return "Its scene objects are";
  }

  if (!others) {
    return `${first} is`;
  }

  return `${first} and ${others} more scene object${others === 1 ? "" : "s"} are`;
}

/**
 * @param objectTools - Every object tool's objects.
 * @param id - A scene object.
 * @returns Its name, or its tool and place among the tool's objects where the listing holds it not.
 */
function nameSceneObject(objectTools: ReadonlyArray<SceneObjectToolDescription>, id: SceneObjectId): string {
  const object: Maybe<SceneObjectDescription> = objectTools
    .find((tool: SceneObjectToolDescription) => tool.tool === id.tool)
    ?.objects.find((it: SceneObjectDescription) => it.id.index === id.index);

  return object?.name ?? `${getSceneToolLabel(id.tool, id.tool)} #${id.index}`;
}
