import { Nullable } from "@xrf/types";

import { LevelSpawnModelFailure } from "@/core/ipc/types/xrf-app";

/** How far a level's spawned models have been read, as data, for everything that reports on them. */
export interface ILevelSpawnReport {
  /** Objects the viewer draws once their visual's model arrives. */
  objects: number;
  /** Visuals they stand as, each read once. */
  visuals: number;
  /** Visuals asked for so far, read or not. */
  read: number;
  /** The visuals that could not be read, whose objects are simply absent from the picture. */
  failures: ReadonlyArray<LevelSpawnModelFailure>;
  /** Why the spawn itself could not be read, which leaves the level without any spawned object; null where it was. */
  failure: Nullable<string>;
}

/** Nothing to read, which is also what a closed level reports. */
export const EMPTY_LEVEL_SPAWN_REPORT: ILevelSpawnReport = {
  failure: null,
  failures: [],
  objects: 0,
  read: 0,
  visuals: 0,
};

/**
 * @param report - What has been read so far.
 * @returns Whether visuals are still to be asked for.
 */
export function isLevelSpawnReading(report: ILevelSpawnReport): boolean {
  return report.read < report.visuals;
}
