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
  SceneSpawnModelDescription,
  SceneToolDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { RenderLoadFailure } from "@xrf/ipc/types/xrf-renderer";
import { assertExhaustive, Maybe } from "@xrf/types";

import { listTextureProblems } from "@/core/level/lib/problems/level-texture-problems";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-report";
import { IEditorProblem } from "@/core/shell/chrome/EditorProblemsPanel";
import { findSceneObject } from "@/tools/level-editor/lib/tools/scene-objects";
import { getSceneToolLabel } from "@/tools/level-editor/lib/tools/scene-tools";

/** What a library object the renderer could not make a model of is listed under. */
export const LEVEL_EDITOR_MODEL_PROBLEM_RULE: string = "Library object not drawn";

/** What a spawn point's model found in the game data and not read by the renderer is listed under. */
export const LEVEL_EDITOR_SPAWN_MODEL_PROBLEM_RULE: string = "Spawn point model not drawn";

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
  /**
   * The models the renderer could not make: a library object's, named by its reference, or a spawn point's, named by
   * its file.
   */
  models: ReadonlyArray<RenderLoadFailure>;
  /** The spawn points drawn as their models, which name those standing as a model the renderer could not make. */
  spawnModels: ReadonlyArray<SceneSpawnModelDescription>;
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
 *   first scene object placing the library object, or the first spawn point standing as the model, and the model, the
 *   others standing as it counted.
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

  return [...parts, ...drawing, ...listTextureProblems(sources.textures), ...listModelProblems(sources)];
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
 * Each model the renderer could not make, with the objects standing as it, which are not drawn as it: a library
 * object's, placed by scene objects, or a spawn point's, found in the game data; a library object whose `.object` the
 * scene's drawing found unreadable is listed there already, by scene object.
 *
 * @param sources - What the problems are listed from: the models, the drawing, the spawn points drawn as models and
 *   every object tool's objects.
 * @returns One row a model, the first object standing as it named and the rest counted.
 */
function listModelProblems(sources: ILevelEditorProblemSources): Array<IEditorProblem> {
  const { models, drawing, spawnModels, objectTools } = sources;
  const listed: ReadonlySet<string> = new Set(drawing.map((problem: SceneDrawProblem) => problem.reference));
  const failed: ReadonlyArray<RenderLoadFailure> = models.filter(
    (failure: RenderLoadFailure) => !listed.has(failure.name)
  );

  if (!failed.length) {
    return [];
  }

  const placing: Map<string, Array<string>> = new Map(failed.map((failure: RenderLoadFailure) => [failure.name, []]));
  const standing: Map<string, Array<string>> = new Map();
  const objects: ReadonlyArray<SceneObjectDescription> =
    objectTools.find((tool: SceneObjectToolDescription) => tool.tool === ESceneToolKind.SCENE_OBJECT)?.objects ?? [];

  for (const object of objects) {
    if (object.properties.kind === ESceneObjectProperties.SCENE_OBJECT) {
      placing.get(object.properties.reference)?.push(object.name);
    }
  }

  for (const model of spawnModels) {
    standing.set(model.model, [...(standing.get(model.model) ?? []), nameSceneObject(objectTools, model.object)]);
  }

  return failed.map(({ name, reason }: RenderLoadFailure): IEditorProblem => {
    const spawnPoints: Maybe<ReadonlyArray<string>> = standing.get(name);
    const [first, ...rest] = spawnPoints ?? placing.get(name) ?? [];

    if (spawnPoints) {
      return {
        message: `${describePlacing(first, rest.length, "spawn point")} not drawn as a model: ${reason}`,
        rule: LEVEL_EDITOR_SPAWN_MODEL_PROBLEM_RULE,
        subject: first ? `${first} · ${name}` : name,
      };
    }

    return {
      message: `${describePlacing(first, rest.length, "scene object")} not drawn: ${reason}`,
      rule: LEVEL_EDITOR_MODEL_PROBLEM_RULE,
      subject: first ? `${first} · ${name}` : name,
    };
  });
}

/**
 * @param first - The first object standing as a model, if the listing holds any.
 * @param others - How many more stand as it.
 * @param noun - What such an object is called.
 * @returns Who is not drawn, as a sentence's subject.
 */
function describePlacing(first: Maybe<string>, others: number, noun: string): string {
  if (!first) {
    return `Its ${noun}s are`;
  }

  if (!others) {
    return `${first} is`;
  }

  return `${first} and ${others} more ${noun}${others === 1 ? "" : "s"} are`;
}

/**
 * @param objectTools - Every object tool's objects.
 * @param id - A scene object.
 * @returns Its name, or its tool and place among the tool's objects where the listing holds it not.
 */
function nameSceneObject(objectTools: ReadonlyArray<SceneObjectToolDescription>, id: SceneObjectId): string {
  return findSceneObject(objectTools, id)?.name ?? `${getSceneToolLabel(id.tool, id.tool)} #${id.index}`;
}
