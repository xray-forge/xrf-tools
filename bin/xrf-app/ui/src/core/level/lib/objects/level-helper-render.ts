import { ELevelObjectId, LEVEL_OBJECT_KINDS, LevelObject, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { RenderHelperOptions } from "@xrf/ipc/types/xrf-renderer";

import { ILevelHelperOptions, isLevelHelperOn } from "@/core/level/lib/objects/level-helper-options";
import { NO_NATIVE_HELPERS } from "@/core/render/lib/native/native-view-options";

/**
 * @param objects - Level objects.
 * @returns The spawn indices of those the spawn places, ascending, as the renderer and the world name them.
 */
export function toLevelSpawnIndices(objects: ReadonlyArray<LevelObject>): Array<number> {
  return objects
    .filter((object: LevelObject) => object.id.source === ELevelObjectId.SPAWN)
    .map((object: LevelObject) => object.id.index)
    .sort((first: number, second: number) => first - second);
}

/**
 * @param helpers - Which helpers the viewer draws.
 * @param selected - The level objects selected, which draw marked and never fade.
 * @param hidden - The level objects hidden, which draw nothing.
 * @returns What the renderer draws of the level's helpers: each kind by its place in the kind table, as the backend
 *   numbers them.
 */
export function toLevelHelperRenderOptions(
  helpers: ILevelHelperOptions,
  selected: ReadonlyArray<LevelObject>,
  hidden: ReadonlyArray<LevelObject>
): RenderHelperOptions {
  return {
    ...NO_NATIVE_HELPERS,
    distanceScale: helpers.distanceScale,
    hidden: toLevelSpawnIndices(hidden),
    isShown: helpers.isShown,
    occludedStrength: helpers.occludedStrength,
    kinds: LEVEL_OBJECT_KINDS.reduce(
      (bits: number, entry: LevelObjectKindEntry, index: number) =>
        isLevelHelperOn(helpers, entry.kind) ? (bits | (1 << index)) >>> 0 : bits,
      0
    ),
    selected: toLevelSpawnIndices(selected),
  };
}
