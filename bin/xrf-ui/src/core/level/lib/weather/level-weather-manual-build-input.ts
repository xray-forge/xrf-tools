import { Nullable } from "@xrf/types";

import { LevelWeatherDescription } from "@/core/ipc/types/xrf-app";
import { XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";

/** What the keyframe set by hand is built from. */
export interface ILevelWeatherManualBuildInput {
  manual: ILevelManualWeather;
  /** The level's weather it plays over, or null where it does not read. */
  description: Nullable<LevelWeatherDescription>;
  /** The engine target, for a level whose weather does not read. */
  engine: XrayEngine;
}
