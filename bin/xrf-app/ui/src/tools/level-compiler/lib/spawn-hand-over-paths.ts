import { getPathDirectory, getPathSeparator } from "@xrf/path";
import { Nullable } from "@xrf/types";

/** The spawn compiler's paths for building a spawn over the level a build wrote. */
export interface ISpawnHandOverPaths {
  /** The directory holding the level's directory, where its neighbours are built too. */
  levelsRoot: string;
  /** `spawns` beside that directory, where a gamedata keeps its spawns. */
  destination: string;
  /** The game data the level was compiled with, whose `configs\game_levels.ltx` lists it; none where none was named. */
  gamedata: Nullable<string>;
}

/**
 * Where the spawn compiler builds a spawn over the level a build wrote, as a gamedata lays its levels and spawns out:
 * the level's neighbours in the directory holding it, the spawn in `spawns` beside that directory, and the level list
 * in the game data the level was compiled with.
 *
 * @param level - The level directory the build wrote.
 * @param gamedata - The game data the compile read, if one was named.
 * @returns The spawn compiler form's paths.
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
