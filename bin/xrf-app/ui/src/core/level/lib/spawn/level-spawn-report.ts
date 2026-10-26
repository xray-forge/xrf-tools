import { Nullable } from "@xrf/types";

/** What the level's spawn came to: whether its objects are listed, how many, and what stopped the listing. */
export interface ILevelSpawnReport {
  /** Whether the spawn's objects are known: nothing is reported before. */
  isListed: boolean;
  /** Objects the viewer draws once their visual's model is read. */
  objects: number;
  /** Visuals they stand as, each read once. */
  visuals: number;
  /** Why the spawn could not be listed, which leaves no spawned object drawn; null where nothing stopped it. */
  failure: Nullable<string>;
}

/** Nothing listed yet, which is also what a closed level reports. */
export const EMPTY_LEVEL_SPAWN_REPORT: ILevelSpawnReport = {
  failure: null,
  isListed: false,
  objects: 0,
  visuals: 0,
};

/**
 * @param report - What the level's spawn came to.
 * @returns Whether its objects are still being listed.
 */
export function isLevelSpawnReading(report: ILevelSpawnReport): boolean {
  return report.failure === null && !report.isListed;
}
