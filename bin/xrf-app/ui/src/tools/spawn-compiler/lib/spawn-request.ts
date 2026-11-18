import { SpawnListedLevel, SpawnRootRequest } from "@xrf/ipc/types/xrf-spawn-builder";
import { Nullable } from "@xrf/types";

/** What the form names for a spawn's build. */
export interface ISpawnBuildChoice {
  /** The game data holding `configs/game_levels.ltx`. */
  gamedata: Nullable<string>;
  /** The directory holding the built levels' directories. */
  levelsRoot: Nullable<string>;
  /** The directory the spawn is written into. */
  destination: Nullable<string>;
  /** The levels ticked, by directory name, in the order the game lists them. */
  levels: ReadonlyArray<string>;
  /** The level the actor starts on; null for the one it stands on. */
  start: Nullable<string>;
  /** The spawn's name, written as `<name>.spawn`. */
  name: string;
  /** Whether a restrictor cutting an AI map apart refuses the build rather than being reported. */
  isSeparatorStrict: boolean;
  /** Whether a spawn of the name already in the directory may be written over. */
  isForced: boolean;
}

/** The row of the form a problem is said beside. */
export type TSpawnBuildField = "levels" | "start" | "name";

/** Why the build the form names cannot run, said beside the row to change. */
export interface ISpawnBuildProblem {
  field: TSpawnBuildField;
  message: string;
}

/**
 * The levels ticked after one level's box changes, kept in the order the listing gives them.
 *
 * @param listed - The levels offered, in the game's order.
 * @param levels - The levels ticked before.
 * @param level - The level whose box changed.
 * @param isChecked - Whether it is now ticked.
 * @returns The levels ticked after.
 */
export function toggleSpawnLevel(
  listed: ReadonlyArray<SpawnListedLevel>,
  levels: ReadonlyArray<string>,
  level: string,
  isChecked: boolean
): ReadonlyArray<string> {
  return listed
    .map((it: SpawnListedLevel) => it.name)
    .filter((name: string) => (name === level ? isChecked : levels.includes(name)));
}

/**
 * Whether the build the form names cannot run: no level ticked, a start level not among them, or a name that is no
 * plain file name. Writing over a spawn already there is left to the build, which alone sees whether it is.
 *
 * @param choice - What the form names.
 * @returns Why it cannot run, or null where it can.
 */
export function getSpawnBuildProblem(choice: ISpawnBuildChoice): Nullable<ISpawnBuildProblem> {
  if (choice.levels.length === 0) {
    return { field: "levels", message: "Tick the levels the spawn is built from" };
  }

  if (choice.start !== null && !choice.levels.includes(choice.start)) {
    return { field: "start", message: "The start level is not one of the levels ticked" };
  }

  const name: string = choice.name.trim();

  if (!name) {
    return { field: "name", message: "Name the spawn" };
  }

  if (/[\\/:*?"<>|]/.test(name) || name === "." || name === "..") {
    return { field: "name", message: "Name the spawn as a plain file name, which is written as <name>.spawn" };
  }

  return null;
}

/**
 * The build request the form names; none while a path is missing or {@link getSpawnBuildProblem} finds a problem.
 *
 * @param choice - What the form names.
 * @returns The request, or null.
 */
export function toSpawnBuildRequest(choice: ISpawnBuildChoice): Nullable<SpawnRootRequest> {
  if (!choice.gamedata || !choice.levelsRoot || !choice.destination || getSpawnBuildProblem(choice)) {
    return null;
  }

  return {
    gamedata: choice.gamedata,
    levelsRoot: choice.levelsRoot,
    levels: [...choice.levels],
    destination: choice.destination,
    name: choice.name.trim(),
    options: { start: choice.start, isSeparatorStrict: choice.isSeparatorStrict },
    isForced: choice.isForced,
  };
}
