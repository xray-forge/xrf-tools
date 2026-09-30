import { Nullable } from "@xrf/types";

import { XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { XrayRoots } from "@/core/ipc/types/xrf-vfs";
import { TLevelRendererWeatherBase } from "@/core/level/lib/weather/level-renderer-weather-base";

/** What the keyframe set by hand is played over. */
export interface ILevelManualWeatherBuilderInput {
  /** What every weather of the level plays with, or null for a level whose weather does not read. */
  base: Nullable<TLevelRendererWeatherBase>;
  /** The engine target, for a level whose weather does not read. */
  engine: XrayEngine;
  /** Roots the level was opened in. */
  roots: XrayRoots;
}
