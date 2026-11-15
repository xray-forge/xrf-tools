import {
  ESceneToolKind,
  SceneDrawDescription,
  SceneObjectsDescription,
  SceneObjectToolDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

/** Why an open scene draws nothing. */
export enum ELevelEditorSceneBlank {
  /** Its scene object tool holds no object: what it holds is helpers alone, which are not drawn. */
  NO_SCENE_OBJECTS = "no-scene-objects",
  /** It places library objects, and none of them was drawn. */
  NOTHING_FOUND = "nothing-found",
}

/**
 * @param objects - The open scene's object tools' objects, or null before they are read.
 * @param drawing - What the open scene's picture needs, or null before it is read.
 * @returns Why the scene draws nothing, or null while either is unread and for a scene that draws an object.
 */
export function toLevelEditorSceneBlank(
  objects: Nullable<SceneObjectsDescription>,
  drawing: Nullable<SceneDrawDescription>
): Nullable<ELevelEditorSceneBlank> {
  if (!objects || !drawing || drawing.bounds) {
    return null;
  }

  const placed: number =
    objects.tools.find((tool: SceneObjectToolDescription) => tool.tool === ESceneToolKind.SCENE_OBJECT)?.objects
      .length ?? 0;

  return placed ? ELevelEditorSceneBlank.NOTHING_FOUND : ELevelEditorSceneBlank.NO_SCENE_OBJECTS;
}
