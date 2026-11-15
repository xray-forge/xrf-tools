import { SceneObjectId, SceneObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderLevelHit, RenderLevelHit, RenderSelection } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { toLevelSpawnSelection } from "@/core/level/lib/pick/level-pick-selection";
import { toSceneRenderId, toSceneRenderObject } from "@/tools/level-editor/lib/render/scene-render-id";
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
 * @returns The scene object it is, a model and a helper alike by the render id it carries: a scene object's model, a
 *   spawn point's model or any helper; null for nothing listed, and for a surface, which a scene draws none of.
 */
export function toLevelEditorPick(
  hit: Nullable<RenderLevelHit>,
  objects: Nullable<SceneObjectsDescription>
): Nullable<SceneObjectId> {
  const id: Nullable<SceneObjectId> =
    hit?.kind === ERenderLevelHit.SPAWN || hit?.kind === ERenderLevelHit.HELPER
      ? toSceneRenderObject(hit.object)
      : null;

  return id && isSceneObjectListed(objects, id) ? id : null;
}

/**
 * @param id - The selected scene object, or null for none.
 * @returns What the viewport outlines of it as selected: the model standing as it, named by its render id, whatever
 *   its tool, which outlines nothing for an object drawn as no model, its helper's own selection lifting that instead;
 *   null for nothing.
 */
export function toLevelEditorSelection(id: Nullable<SceneObjectId>): Nullable<RenderSelection> {
  const object: Nullable<number> = id ? toSceneRenderId(id) : null;

  return object === null ? null : toLevelSpawnSelection(object);
}
