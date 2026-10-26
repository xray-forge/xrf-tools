import { ILevelWeatherMemory } from "@/core/level/lib/weather/level-weather-memory";

/**
 * What a level's weather played when something borrowed it, a level map's capture playing its own: the cycle, what
 * lit the level, the clock's control and where it stood, which `LevelWeatherService.release` plays again.
 */
export type ILevelWeatherHold = Pick<ILevelWeatherMemory, "cycle" | "source" | "control" | "time">;
