import { LevelWeatherCycle, LevelWeatherDescription } from "@/core/ipc/types/xrf-app";

/** What a cycle is built from. */
export interface ILevelWeatherCycleBuildInput {
  cycle: LevelWeatherCycle;
  /** The level's weather it plays over. */
  description: LevelWeatherDescription;
}
