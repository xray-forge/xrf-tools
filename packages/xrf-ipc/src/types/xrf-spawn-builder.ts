// Auto-generated rust bindings. Do not edit it manually.

import { Report } from "#/types/xrf-report";

/** A level the game lists, as a front-end offers it for a build. */
export type SpawnListedLevel = {
  /** Its directory name, as a request names it. */
  name: string;
  /** The name a graph point names it by in a connection. */
  caption: string;
  id: number;
  /** Whether its directory holds `level.ai`, `level.spawn` and `level.game`. */
  isComplete: boolean;
};

/** How a root spawn is built beyond its levels. */
export type SpawnRootOptions = {
  /** The level the actor starts on, if not its own: xrAI's `-start`. */
  start: string | null;
  /**
   * Refuse, rather than report, a restrictor cutting an AI map apart: xrAI's check without `-no_separator_check`,
   * kept to real cuts.
   */
  isSeparatorStrict: boolean;
};

/** What a root spawn build from a gamedata root wrote. */
export type SpawnRootOutcome = {
  /** The spawn file written. */
  path: string;
  /** The levels built, in id order. */
  levels: Array<string>;
  guid: string;
  graphGuid: string;
  objects: number;
  artefactPoints: number;
  patrols: number;
  report: Report;
};

/** A root spawn to build from a gamedata root's level list and the levels' directories, written as `<name>.spawn`. */
export type SpawnRootRequest = {
  /** The root holding `configs/game_levels.ltx`, and by default the levels and the spawns. */
  gamedata: string;
  /** The directory holding the levels' directories; none for `levels/` under `gamedata`. */
  levelsRoot: string | null;
  /**
   * The levels to build, by directory name; none builds every level `[levels]` lists whose directory holds
   * `level.ai`, `level.spawn` and `level.game`.
   */
  levels: Array<string> | null;
  /** The directory the spawn is written into; none for `spawns/` under `gamedata`. */
  destination: string | null;
  /** The spawn's name, written as `<name>.spawn`. */
  name: string;
  options: SpawnRootOptions;
  /** Write over a spawn already there in any case of its name; unforced, the build refuses before reading a level. */
  isForced: boolean;
};
