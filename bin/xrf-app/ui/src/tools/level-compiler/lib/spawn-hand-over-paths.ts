import { getPathDirectory, getPathSeparator } from "@xrf/path";
import { Nullable } from "@xrf/types";

/** The spawn compiler's paths for building a spawn over the level a build wrote. */
export interface ISpawnHandOverPaths {
  /** Directory holding the level's directory, where its neighbours are built too. */
  levelsRoot: string;
  /** `spawns` beside the levels root, as a gamedata lays it out. */
  destination: string;
  /** Game data the level was compiled with, whose `configs\game_levels.ltx` lists it; null where none was named. */
  gamedata: Nullable<string>;
}

/**
 * The spawn compiler form's paths for a level directory a build wrote.
 */
export function getSpawnHandOverPaths(level: string, gamedata: Nullable<string>): ISpawnHandOverPaths {
  const separator: string = getPathSeparator(level);
  const levelsRoot: string = getPathDirectory(level.replace(/[\\/]+$/, ""));

  return {
    levelsRoot,
    destination: [getPathDirectory(levelsRoot), "spawns"].join(separator),
    gamedata,
  };
}
