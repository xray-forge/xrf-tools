import {
  SCENE_HELPER_INDEX_BITS,
  SCENE_HELPER_TOOLS,
  SceneHelperToolEntry,
  SceneObjectId,
} from "@xrf/ipc/types/xrf-app-levels";
import { Nullable } from "@xrf/types";

/** What an index among its tool's objects may reach in a helper's id: the bits below the tool's place. */
const INDEX_MASK: number = 2 ** SCENE_HELPER_INDEX_BITS - 1;

/** How far a tool's place stands above the index. */
const TOOL_SCALE: number = 2 ** SCENE_HELPER_INDEX_BITS;

/**
 * @param object - A scene object.
 * @returns The id its helper carries, as the backend's `SceneHelperId` makes it: its tool's place, the table's `kind`,
 *   above its index among the tool's objects; null for a tool drawing no helper or an index past what an id holds.
 */
export function toSceneHelperId(object: SceneObjectId): Nullable<number> {
  const entry: Nullable<SceneHelperToolEntry> =
    SCENE_HELPER_TOOLS.find((it: SceneHelperToolEntry) => it.tool === object.tool) ?? null;

  return entry && object.index >= 0 && object.index <= INDEX_MASK ? entry.kind * TOOL_SCALE + object.index : null;
}

/**
 * @param objects - Scene objects.
 * @returns The ids of the helpers of those that draw one, ascending, as the renderer searches the lists it is given.
 */
export function toSceneHelperIds(objects: ReadonlyArray<SceneObjectId>): Array<number> {
  return objects
    .map(toSceneHelperId)
    .filter((id: Nullable<number>): id is number => id !== null)
    .sort((first: number, second: number) => first - second);
}

/**
 * @param id - The id a scene helper carries, as a pick of it answers.
 * @returns The scene object it stands for; null for an id no helped tool's place answers.
 */
export function toSceneHelperObject(id: number): Nullable<SceneObjectId> {
  const kind: number = Math.floor(id / TOOL_SCALE);
  const entry: Nullable<SceneHelperToolEntry> =
    SCENE_HELPER_TOOLS.find((it: SceneHelperToolEntry) => it.kind === kind) ?? null;

  return entry ? { index: id % TOOL_SCALE, tool: entry.tool } : null;
}
