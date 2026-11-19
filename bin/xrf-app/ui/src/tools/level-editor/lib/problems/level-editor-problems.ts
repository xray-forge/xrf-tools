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
import { IEditorProblem } from "@xrf/ui/editor/problems";

import { listTextureProblems } from "@/core/level/lib/problems/level-texture-problems";
import { ILevelTextureProblem } from "@/core/level/lib/texture/level-texture-report";
import { findSceneObject } from "@/tools/level-editor/lib/tools/scene-objects";
import { getSceneToolLabel } from "@/tools/level-editor/lib/tools/scene-tools";

/** The rule for a library object the renderer could not make a model of. */
export const LEVEL_EDITOR_MODEL_PROBLEM_RULE: string = "Library object not drawn";

/** The rule for a spawn point model found in the game data that the renderer could not read. */
export const LEVEL_EDITOR_SPAWN_MODEL_PROBLEM_RULE: string = "Spawn point model not drawn";

/**
 * What a scene's problems are listed from: the open, each part's read, and the renderer once resident.
 */
export interface ILevelEditorProblemSources {
  /** Every tool the scene holds part files of, naming the tool a problem is on. */
  tools: ReadonlyArray<SceneToolDescription>;
  /** What the open found of the scene's part files. */
  scene: ReadonlyArray<SceneProblem>;
  /** The object tools whose objects did not read. */
  objects: ReadonlyArray<SceneProblem>;
  /** The binary tools whose parts did not read. */
  binaryTools: ReadonlyArray<SceneProblem>;
  /** The scene objects the scene's picture does not draw. */
  drawing: ReadonlyArray<SceneDrawProblem>;
  /** Every object tool's objects, naming the objects that drawing and model problems are on. */
  objectTools: ReadonlyArray<SceneObjectToolDescription>;
  /** What the renderer's texture set could not answer for properly. */
  textures: ReadonlyArray<ILevelTextureProblem>;
  /** Models the renderer could not make: a library object's by reference, or a spawn point's by file. */
  models: ReadonlyArray<RenderLoadFailure>;
  /** The spawn points drawn as models, naming those standing as a failed model. */
  spawnModels: ReadonlyArray<SceneSpawnModelDescription>;
}

/**
 * What the SDK's Level Editor would skip or refuse of a scene, and what is not drawn, one row a problem.
 *
 * Rows run: the open's findings, the object and binary tool reads, undrawn scene objects, then texture and model
 * failures. A part's subject is its tool's label and part file; a drawing problem's, the object and its library
 * object; a model problem's, the first object standing as the model, the others counted.
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
 * The rule name for why the editor would skip or refuse a part.
 */
export function describeSceneProblemKind(kind: SceneProblemKind): string {
  switch (kind) {
    case ESceneProblemKind.UNREADABLE:
      return "Unreadable";

    case ESceneProblemKind.WRONG_VERSION:
      return "Wrong tool version";

    case ESceneProblemKind.FOREIGN_CLASS:
      return "Another tool's class";

    default:
      return assertExhaustive(kind);
  }
}

/**
 * The rule name for why a scene object is not drawn.
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
 * One row per model the renderer could not make, naming the first object standing as it and counting the rest.
 *
 * Skips a library object the drawing already lists by scene object.
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
 * The objects not drawn as a model, as a sentence's subject: `first` and a count of `others`.
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
 * A scene object's name, or its tool and index where the listing lacks it.
 */
function nameSceneObject(objectTools: ReadonlyArray<SceneObjectToolDescription>, id: SceneObjectId): string {
  return findSceneObject(objectTools, id)?.name ?? `${getSceneToolLabel(id.tool, id.tool)} #${id.index}`;
}
