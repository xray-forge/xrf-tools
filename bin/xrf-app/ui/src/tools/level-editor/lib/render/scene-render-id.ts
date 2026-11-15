import {
  SCENE_RENDER_INDEX_BITS,
  SCENE_RENDER_TOOLS,
  SceneObjectId,
  SceneToolKind,
} from "@xrf/ipc/types/xrf-app-levels";
import { Maybe, Nullable } from "@xrf/types";

/** What an index among its tool's objects may reach in a render id: the bits below the tool's place. */
const INDEX_MASK: number = 2 ** SCENE_RENDER_INDEX_BITS - 1;

/** How far a tool's place stands above the index. */
const TOOL_SCALE: number = 2 ** SCENE_RENDER_INDEX_BITS;

/**
 * @param object - A scene object.
 * @returns The one id it is named to the renderer by, whatever it draws, as the backend's `SceneRenderId` makes it: its
 *   tool's place among the editor's tools above its index among the tool's objects; null for an index past what an id
 *   holds.
 */
export function toSceneRenderId(object: SceneObjectId): Nullable<number> {
  const place: number = SCENE_RENDER_TOOLS.indexOf(object.tool);

  return place >= 0 && object.index >= 0 && object.index <= INDEX_MASK ? place * TOOL_SCALE + object.index : null;
}

/**
 * @param objects - Scene objects.
 * @returns Their render ids, ascending, as the renderer searches the lists it is given.
 */
export function toSceneRenderIds(objects: ReadonlyArray<SceneObjectId>): Array<number> {
  return objects
    .map(toSceneRenderId)
    .filter((id: Nullable<number>): id is number => id !== null)
    .sort((first: number, second: number) => first - second);
}

/**
 * @param id - A render id, as a pick of a model or a helper answers it.
 * @returns The scene object it names; null for an id no tool's place answers.
 */
export function toSceneRenderObject(id: number): Nullable<SceneObjectId> {
  const tool: Maybe<SceneToolKind> = id >= 0 ? SCENE_RENDER_TOOLS[Math.floor(id / TOOL_SCALE)] : undefined;

  return tool === undefined ? null : { index: id % TOOL_SCALE, tool };
}
