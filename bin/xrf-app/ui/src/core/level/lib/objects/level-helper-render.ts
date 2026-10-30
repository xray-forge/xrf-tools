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
 * @param listed - The level's objects as listed, by whose places the backend names their helpers.
 * @returns What the renderer draws of the level's helpers: each kind by its place in the kind table, as the backend
 *   numbers them.
 */
export function toLevelHelperRenderOptions(
  helpers: ILevelHelperOptions,
  targets: ILevelHelperTargets,
  listed: ReadonlyArray<LevelObject>
): RenderHelperOptions {
  return {
    ...NO_NATIVE_HELPERS,
    distanceScale: helpers.distanceScale,
    hidden: toLevelListingPlaces(targets.hidden, listed),
    hovered: (targets.hovered ? toLevelListingPlaces([targets.hovered], listed)[0] : null) ?? null,
    isRestFaded: helpers.isRestFaded,
    isShown: helpers.isShown,
    kinds: LEVEL_OBJECT_KINDS.reduce(
      (bits: number, entry: LevelObjectKindEntry, index: number) =>
        isLevelHelperOn(helpers, entry.kind) ? (bits | (1 << index)) >>> 0 : bits,
      0
    ),
    occludedStrength: helpers.occludedStrength,
    selected: toLevelListingPlaces(targets.selected, listed),
  };
}
