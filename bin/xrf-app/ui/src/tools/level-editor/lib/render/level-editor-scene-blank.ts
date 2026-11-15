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
  /** It places library objects, none of them was drawn, and it holds nothing a helper is drawn for. */
  NOTHING_FOUND = "nothing-found",
  /** It holds objects, of tools that draw neither a model nor a helper: sectors, groups and the like. */
  NOTHING_DRAWN = "nothing-drawn",
  /** It draws helpers alone, and the Show action has every one of them switched off. */
  HELPERS_HIDDEN = "helpers-hidden",
}

/**
 * @param objects - The open scene's object tools' objects, or null before they are read.
 * @param drawing - What the open scene's picture needs, or null before it is read.
 * @param helpers - What the page needs of the open scene's helpers, or null before it is read.
 * @param shown - Which helpers the Show action has switched on.
 * @returns Why the scene draws nothing, or null while any is unread and for a scene that draws an object or a helper.
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
