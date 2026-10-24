import { ESessionRootChange, LEVEL_GEOMETRY_PART, LEVEL_SURFACES_PART, SessionChange } from "@/core/ipc/types/xrf-app";

/**
 * The parts a level's sectors are packed and drawn from: a change to either streams its scene afresh, as the backend's
 * `LevelRenderSource::is_streamed_afresh` decides.
 */
const LEVEL_SCENE_PARTS: ReadonlyArray<string> = [LEVEL_GEOMETRY_PART, LEVEL_SURFACES_PART];

/**
 * @param change - A change to a level's session.
 * @returns Whether its scene streams afresh, as a new scene: the level was derived anew, or what its sectors are packed
 *   and drawn from went stale.
 */
export function isLevelSceneStreamedAfresh(change: SessionChange): boolean {
  return (
    change.root.kind === ESessionRootChange.DERIVED ||
    (change.root.kind === ESessionRootChange.KEPT &&
      change.staleParts.some((part: string) => LEVEL_SCENE_PARTS.includes(part)))
  );
}
