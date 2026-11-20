import {
  SCENE_RENDER_COLLISION_BIT,
  SCENE_RENDER_EDITOR_BIT,
  SCENE_RENDER_INDEX_BITS,
  SCENE_RENDER_TOOLS,
  SceneObjectId,
  SceneToolKind,
} from "@xrf/ipc/types/xrf-app-levels";
import { Maybe, Nullable } from "@xrf/types";

/** The largest object index a render id holds: the bits below the tool's place. */
const INDEX_MASK: number = 2 ** SCENE_RENDER_INDEX_BITS - 1;

/** The multiplier placing a tool's place above the index. */
const TOOL_SCALE: number = 2 ** SCENE_RENDER_INDEX_BITS;

/** The bits above every tool's place that name an object's other models, highest first. */
const PART_BITS: ReadonlyArray<number> = [SCENE_RENDER_COLLISION_BIT, SCENE_RENDER_EDITOR_BIT];

/**
 * The id a scene object is named to the renderer by, as the backend's `SceneRenderId` makes it: its tool's place above
 * its index.
 *
 * @returns The id, or null for an index past what an id holds.
 */
export function toSceneRenderId(object: SceneObjectId): Nullable<number> {
  const place: number = SCENE_RENDER_TOOLS.indexOf(object.tool);

  return place >= 0 && object.index >= 0 && object.index <= INDEX_MASK ? place * TOOL_SCALE + object.index : null;
}

/**
 * Scene objects' render ids, ascending, as the renderer searches the lists it is given.
 */
export function toSceneRenderIds(objects: ReadonlyArray<SceneObjectId>): Array<number> {
  return objects
    .map(toSceneRenderId)
    .filter((id: Nullable<number>): id is number => id !== null)
    .sort((first: number, second: number) => first - second);
}

/**
 * The render ids of every model scene objects stand as, ascending: each object's own id, and that id with the
 * collision and editor-only bits set. An id with no model names nothing.
 */
export function toSceneRenderModelIds(objects: ReadonlyArray<SceneObjectId>): Array<number> {
  return toSceneRenderIds(objects)
    .flatMap((id: number) => [id, ...PART_BITS.map((bit: number) => id + bit)])
    .sort((first: number, second: number) => first - second);
}

/**
 * The scene object a picked render id names, from any of its models or its helper.
 *
 * @returns The object, or null for an id no tool's place answers.
 */
export function toSceneRenderObject(id: number): Nullable<SceneObjectId> {
  const own: number = PART_BITS.reduce(
    (stripped: number, bit: number) => (stripped >= bit ? stripped - bit : stripped),
    id
  );
  const tool: Maybe<SceneToolKind> = own >= 0 ? SCENE_RENDER_TOOLS[Math.floor(own / TOOL_SCALE)] : undefined;

  return tool === undefined ? null : { index: own % TOOL_SCALE, tool };
}
