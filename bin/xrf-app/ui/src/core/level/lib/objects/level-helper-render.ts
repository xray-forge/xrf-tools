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
 * @param listed - The level's objects as listed.
 * @returns Each object's place in the listing by its key: the id the backend gives its helper.
 */
export function toLevelListingPlaceMap(listed: ReadonlyArray<LevelObject>): ReadonlyMap<string, number> {
  return new Map(listed.map((object: LevelObject, index: number) => [toLevelObjectKey(object.id), index]));
}

/**
 * @param objects - Level objects.
 * @param places - Each listed object's place in the listing by its key (`toLevelListingPlaceMap`).
 * @returns The places of those listed among the listing, ascending: the ids the backend gives their helpers.
 */
export function toLevelListingPlaces(
  objects: ReadonlyArray<LevelObject>,
  places: ReadonlyMap<string, number>
): Array<number> {
  return toAscending(
    objects
      .map((object: LevelObject) => places.get(toLevelObjectKey(object.id)))
      .filter((place: number | undefined): place is number => place !== undefined)
  );
}

/** The level objects a viewport singles out for its helpers. */
export interface ILevelHelperTargets {
  /** Selected, which draw lifted and never fade, their models boxed. */
  selected: ReadonlyArray<LevelObject>;
  /** Hidden, which draw nothing. */
  hidden: ReadonlyArray<LevelObject>;
  /** Under the pointer, which draws lifted half as far and never fades, its model boxed. */
  hovered: Nullable<LevelObject>;
}

/**
 * @param helpers - Which helpers the viewer draws.
 * @param targets - The level objects the viewport singles out.
 * @param places - Each listed object's place in the listing by its key, by which the backend names its helper.
 * @returns What the renderer draws of the level's helpers: each kind by its place in the kind table, as the backend
 *   numbers them.
 */
export function toLevelHelperRenderOptions(
  helpers: ILevelHelperOptions,
  targets: ILevelHelperTargets,
  places: ReadonlyMap<string, number>
): RenderHelperOptions {
  return {
    ...NO_NATIVE_HELPERS,
    distanceScale: helpers.distanceScale,
    hidden: toLevelListingPlaces(targets.hidden, places),
    hovered: targets.hovered ? (places.get(toLevelObjectKey(targets.hovered.id)) ?? null) : null,
    isRestFaded: helpers.isRestFaded,
    isShown: helpers.isShown,
    kinds: LEVEL_OBJECT_KINDS.reduce(
      (bits: number, entry: LevelObjectKindEntry, index: number) =>
        isLevelHelperOn(helpers, entry.kind) ? (bits | (1 << index)) >>> 0 : bits,
      0
    ),
    occludedStrength: helpers.occludedStrength,
    selected: toLevelListingPlaces(targets.selected, places),
  };
}
