import { SceneObjectId, SceneObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderLevelHit, RenderLevelHit, RenderSelection } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { toLevelSpawnSelection } from "@/core/level/lib/pick/level-pick-selection";
import { toSceneRenderId, toSceneRenderObject } from "@/tools/level-editor/lib/render/scene-render-id";
import { findSceneObject } from "@/tools/level-editor/lib/tools/scene-objects";

/**
 * Whether the open scene's object listing holds an object.
 */
export function isSceneObjectListed(objects: Nullable<SceneObjectsDescription>, id: SceneObjectId): boolean {
  return objects !== null && findSceneObject(objects.tools, id) !== null;
}

/**
 * The listed scene object a viewport hit names by its render id, from a model or a helper alike.
 *
 * @returns The object, or null for a surface or an object not listed.
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
 * The viewport selection outlining a selected object's model by its render id.
 *
 * An object drawn as no model gets no outline; its helper's own selection lifts it instead.
 */
export function toLevelEditorSelection(id: Nullable<SceneObjectId>): Nullable<RenderSelection> {
  const object: Nullable<number> = id ? toSceneRenderId(id) : null;

  return object === null ? null : toLevelSpawnSelection(object);
}
