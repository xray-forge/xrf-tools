import { LEVEL_OBJECT_KINDS, LevelObject, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";
import { RenderHelperOptions } from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { ILevelHelperOptions, isLevelHelperOn } from "@/core/level/lib/objects/level-helper-options";
import { getLevelSpawnIndex, toLevelObjectKey } from "@/core/level/lib/objects/level-object-kinds";
import { NO_NATIVE_HELPERS } from "@/core/render/lib/native/native-view-options";

/** Ascending, as the renderer and the world search the lists they are given. */
function toAscending(values: Array<number>): Array<number> {
  return values.sort((first: number, second: number) => first - second);
}

/**
 * @param objects - Level objects.
 * @returns The spawn indices of those the spawn places, ascending, as the world names the models it stands.
 */
export function toLevelSpawnIndices(objects: ReadonlyArray<LevelObject>): Array<number> {
  return toAscending(
    objects
      .map((object: LevelObject) => getLevelSpawnIndex(object.id))
      .filter((index: Nullable<number>): index is number => index !== null)
  );
}

/**
 * @param objects - Level objects.
 * @param listed - The level's objects as listed.
 * @returns The places of those listed among the listing, ascending: the ids the backend gives their helpers.
 */
export function toLevelListingPlaces(
  objects: ReadonlyArray<LevelObject>,
  listed: ReadonlyArray<LevelObject>
): Array<number> {
  const places: ReadonlyMap<string, number> = new Map(
    listed.map((object: LevelObject, index: number) => [toLevelObjectKey(object.id), index])
  );

  return toAscending(
    objects
      .map((object: LevelObject) => places.get(toLevelObjectKey(object.id)))
      .filter((place: number | undefined): place is number => place !== undefined)
  );
}

/**
 * @param helpers - Which helpers the viewer draws.
 * @param selected - The level objects selected, which draw marked and never fade.
 * @param hidden - The level objects hidden, which draw nothing.
 * @param listed - The level's objects as listed, by whose places the backend names their helpers.
 * @returns What the renderer draws of the level's helpers: each kind by its place in the kind table, as the backend
 *   numbers them.
 */
export function toLevelHelperRenderOptions(
  helpers: ILevelHelperOptions,
  selected: ReadonlyArray<LevelObject>,
  hidden: ReadonlyArray<LevelObject>,
  listed: ReadonlyArray<LevelObject>
): RenderHelperOptions {
  return {
    ...NO_NATIVE_HELPERS,
    distanceScale: helpers.distanceScale,
    hidden: toLevelListingPlaces(hidden, listed),
    isShown: helpers.isShown,
    kinds: LEVEL_OBJECT_KINDS.reduce(
      (bits: number, entry: LevelObjectKindEntry, index: number) =>
        isLevelHelperOn(helpers, entry.kind) ? (bits | (1 << index)) >>> 0 : bits,
      0
    ),
    occludedStrength: helpers.occludedStrength,
    selected: toLevelListingPlaces(selected, listed),
  };
}
