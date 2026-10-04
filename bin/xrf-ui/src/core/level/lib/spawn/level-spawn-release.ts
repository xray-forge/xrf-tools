import { ELevelSpawnRelease, LevelSpawnRelease } from "@/core/ipc/types/xrf-app";

/** Why a new game releases an object, as the panel says it: the config and section naming it, and what it then does. */
const RELEASE_REASONS: Readonly<Record<LevelSpawnRelease, string>> = {
  [ELevelSpawnRelease.REMOVE_OBJECTS]: "Removed by new_game_setup.ltx [remove_objects]",
  [ELevelSpawnRelease.REPLACE_ITEMS]:
    "Replaced by dynamic_item_spawn.ltx [replace_items]; the replacement is not placed",
};

/**
 * @param release - Why a new game releases an object.
 * @returns The reason as the panel states it.
 */
export function describeLevelSpawnRelease(release: LevelSpawnRelease): string {
  return RELEASE_REASONS[release];
}
