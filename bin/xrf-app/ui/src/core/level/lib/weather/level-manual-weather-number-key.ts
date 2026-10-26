import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";

/** A number of a keyframe set by hand that is offered on a slider, between its limits. */
export type TLevelManualWeatherNumberKey = keyof typeof LEVEL_MANUAL_WEATHER_LIMITS;
