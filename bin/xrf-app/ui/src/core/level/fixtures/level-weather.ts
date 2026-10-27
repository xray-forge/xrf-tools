import { SessionSnapshot } from "@xrf/ipc/types/xrf-app-core";
import { SelectedLevelDescription } from "@xrf/ipc/types/xrf-app-levels";
import { WeatherDescriptor } from "@xrf/ipc/types/xrf-environment";
import { EWorldWeatherPlay, WorldWeatherPlay } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { mockHeldSessionRoot, mockLetGoSessionRoot } from "@/core/session/fixtures/session-root.mocks";
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

/**
 * Opens a level's weather as its loader tells it of one: the loader holds the level, or none, then the weather hears.
 *
 * @param service - The weather to open.
 * @param selected - The level open, or null for none.
 * @returns Once the weather settled.
 */
export async function mockOpenLevelWeather(
  service: LevelWeatherService,
  selected: Nullable<SessionSnapshot<SelectedLevelDescription>>
): Promise<void> {
  const loader: LevelLoadService = service["loadService"];

  if (selected) {
    mockHeldSessionRoot(loader.selection, selected);
  } else {
    mockLetGoSessionRoot(loader.selection);
  }

  // What the weather's own follow of the loader calls once provisioned, which these tests leave unprovisioned.
  await service["open"](selected);
}

/** The keyframe set by hand a play hands over, or null for a play of anything else. */
export function toWeatherKeyframe(play: Nullable<WorldWeatherPlay>): Nullable<WeatherDescriptor> {
  return play?.kind === EWorldWeatherPlay.KEYFRAME ? play.keyframe : null;
}
