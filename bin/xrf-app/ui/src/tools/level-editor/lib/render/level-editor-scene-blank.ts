import {
  ESceneToolKind,
  SceneDrawDescription,
  SceneHelpersDescription,
  SceneHelperToolCount,
  SceneObjectsDescription,
  SceneObjectToolDescription,
} from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

import {
  ILevelEditorHelperOptions,
  isLevelEditorHelperOn,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";

/** Why an open scene draws nothing. */
export enum ELevelEditorSceneBlank {
  /** None of its object tools holds an object. */
  EMPTY = "empty",
  /** It places library objects, none was drawn, and nothing it holds draws a helper. */
  NOTHING_FOUND = "nothing-found",
  /** Its objects are of tools drawing neither a model nor a helper: sectors, groups and the like. */
  NOTHING_DRAWN = "nothing-drawn",
  /** It draws helpers alone, all switched off in the Show action. */
  HELPERS_HIDDEN = "helpers-hidden",
}

/**
 * Why the open scene draws nothing.
 *
 * @returns The reason, or null while any part is unread or where the scene draws an object or a helper.
 */
export function toLevelEditorSceneBlank(
  objects: Nullable<SceneObjectsDescription>,
  drawing: Nullable<SceneDrawDescription>,
  helpers: Nullable<SceneHelpersDescription>,
  shown: ILevelEditorHelperOptions
): Nullable<ELevelEditorSceneBlank> {
  if (!objects || !drawing || !helpers || drawing.bounds) {
    return null;
  }

  if (helpers.tools.length) {
    const isHelped: boolean =
      shown.isShown &&
      helpers.tools.some((counted: SceneHelperToolCount) => isLevelEditorHelperOn(shown, counted.tool));

    return isHelped ? null : ELevelEditorSceneBlank.HELPERS_HIDDEN;
  }

  const placed: number =
    objects.tools.find((tool: SceneObjectToolDescription) => tool.tool === ESceneToolKind.SCENE_OBJECT)?.objects
      .length ?? 0;

  if (placed) {
    return ELevelEditorSceneBlank.NOTHING_FOUND;
  }

  return objects.tools.some((tool: SceneObjectToolDescription) => tool.objects.length)
    ? ELevelEditorSceneBlank.NOTHING_DRAWN
    : ELevelEditorSceneBlank.EMPTY;
}
