import {
  ESessionRootChange,
  LEVEL_GEOMETRY_PART,
  LEVEL_MAP_PART,
  LEVEL_START_PART,
  LEVEL_SURFACES_PART,
  LEVEL_TEXTURES_PART,
  SessionChange,
} from "@/core/ipc/types/xrf-app";

/**
 * The parts a level's sectors are packed and drawn from: a change to either streams its scene afresh, as the backend's
 * `LevelRenderSource::is_streamed_afresh` decides.
 */
const LEVEL_SCENE_PARTS: ReadonlyArray<string> = [LEVEL_GEOMETRY_PART, LEVEL_SURFACES_PART];

/**
 * The parts the page reads a level again for, beside the level itself: where it starts, its map, its surfaces and
 * their textures, which its description shows, and its geometry, which the scene it is shown in streams from.
 */
const LEVEL_READ_PARTS: ReadonlyArray<string> = [
  LEVEL_START_PART,
  LEVEL_MAP_PART,
  LEVEL_TEXTURES_PART,
  ...LEVEL_SCENE_PARTS,
];

/**
 * @param change - A change to a level's session.
 * @param parts - The parts that count.
 * @returns Whether the level was derived anew, or kept with one of `parts` stale.
 */
function isLevelMoved(change: SessionChange, parts: ReadonlyArray<string>): boolean {
  return (
    change.root.kind === ESessionRootChange.DERIVED ||
    (change.root.kind === ESessionRootChange.KEPT && change.staleParts.some((part: string) => parts.includes(part)))
  );
}

/**
 * @param change - A change to a level's session.
 * @returns Whether the page reads the level again: it was derived anew, or a part its description shows or its scene
 *   streams from went stale.
 */
export function isLevelReadAgain(change: SessionChange): boolean {
  return isLevelMoved(change, LEVEL_READ_PARTS);
}

/**
 * @param change - A change to a level's session.
 * @returns Whether its scene streams afresh, as a new scene: the level was derived anew, or what its sectors are packed
 *   and drawn from went stale.
 */
export function isLevelSceneStreamedAfresh(change: SessionChange): boolean {
  return isLevelMoved(change, LEVEL_SCENE_PARTS);
}
