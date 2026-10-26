import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { WeatherDescriptor } from "@xrf/ipc/types/xrf-environment";
import { EWorldWeatherPlay, WorldWeatherPlay } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { mockContainer } from "@/fixtures/utils/container";

/** The level the weather tests open. */
export const LEVEL_WEATHER_SELECTED: SessionSnapshot<SelectedLevelDescription> = {
  sessionId: "level",
  revision: 0,
  value: mockSelectedLevelDescription(),
};

/** A weather service over a level load service of its own. */
export function createLevelWeatherService(): LevelWeatherService {
  return mockContainer([LevelLoadService, LevelWeatherService]).get(LevelWeatherService);
}

/** The keyframe set by hand a play hands over, or null for a play of anything else. */
export function toWeatherKeyframe(play: Nullable<WorldWeatherPlay>): Nullable<WeatherDescriptor> {
  return play?.kind === EWorldWeatherPlay.KEYFRAME ? play.keyframe : null;
}
