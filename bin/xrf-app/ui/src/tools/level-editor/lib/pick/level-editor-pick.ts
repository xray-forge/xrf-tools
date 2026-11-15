import { ESceneToolKind, SceneObjectId, SceneObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderLevelHit, RenderLevelHit, RenderSelection } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { toLevelSpawnSelection } from "@/core/level/lib/pick/level-pick-selection";
import { toSceneHelperObject } from "@/tools/level-editor/lib/helpers/scene-helper-id";
import { findSceneObject } from "@/tools/level-editor/lib/tools/scene-objects";

/**
 * @param objects - The open scene's objects as listed.
 * @param id - A scene object, by its tool and index.
 * @returns Whether the listing holds it.
 */
export function isSceneObjectListed(objects: Nullable<SceneObjectsDescription>, id: SceneObjectId): boolean {
  return objects !== null && findSceneObject(objects.tools, id) !== null;
}

/**
 * @param hit - What a native viewport named under a point of the open scene.
 * @param objects - The scene's objects as listed, which what is picked must be one of.
 * @returns The scene object it is: a model by the index the scene's source named it with, its place among the scene
 *   objects, a helper by the tool and index its id carries; null for nothing listed, and for a surface, which a scene
 *   draws none of.
 */
export function toLevelEditorPick(
  hit: Nullable<RenderLevelHit>,
  objects: Nullable<SceneObjectsDescription>
): Nullable<SceneObjectId> {
  let id: Nullable<SceneObjectId> = null;

  if (hit?.kind === ERenderLevelHit.SPAWN) {
    id = { index: hit.object, tool: ESceneToolKind.SCENE_OBJECT };
  } else if (hit?.kind === ERenderLevelHit.HELPER) {
    id = toSceneHelperObject(hit.object);
  }

  return id && isSceneObjectListed(objects, id) ? id : null;
}

/**
 * @param id - The selected scene object, or null for none.
 * @returns What the viewport outlines of it as selected: a scene object's model; null for nothing, and for an object
 *   drawn as a helper, which the helpers' own selection lifts instead.
 */
export function toLevelEditorSelection(id: Nullable<SceneObjectId>): Nullable<RenderSelection> {
  return id?.tool === ESceneToolKind.SCENE_OBJECT ? toLevelSpawnSelection(id.index) : null;
}
