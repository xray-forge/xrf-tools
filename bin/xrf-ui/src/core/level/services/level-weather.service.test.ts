import { afterEach, describe, expect, it } from "@jest/globals";
import { ERendererTextureEncoding, ERendererWeatherEngine, IRendererWeather } from "@xrf/renderer";
import { Nullable } from "@xrf/types";

import { SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { WeatherCycleId } from "@/core/ipc/types/xrf-environment";
import { LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import {
  readLevelWeatherMemory,
  toLevelWeatherMemoryKey,
  writeLevelWeatherMemory,
} from "@/core/level/lib/weather/level-weather-memory";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherCycle, mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";

const SELECTED: SessionSnapshot<SelectedLevelDescription> = {
  sessionId: "level",
  value: mockSelectedLevelDescription(),
};

function createService(): LevelWeatherService {
  return mockContainer([LevelWeatherService]).get(LevelWeatherService);
}

afterEach(() => {
  resetMockInvoke();
  window.localStorage.clear();
});

describe("LevelWeatherService", () => {
  it("plays the first cycle the level offers, with where each resolved sky is fetched from", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(
        mockLevelWeatherDescription({
          engine: EXrayEngine.EXTENDED,
          sunTable: [{ altitude: 15, longitude: null }],
        })
      ),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    const weather: Nullable<IRendererWeather> = service.weather;

    expect(service.cycle?.name).toBe("default_clear");
    expect(weather?.engine).toBe(ERendererWeatherEngine.EXTENDED);
    expect(weather?.keyframes.map((keyframe) => keyframe.time)).toEqual([0, LEVEL_WEATHER_NOON]);
    expect(weather?.sunTable).toEqual([{ altitude: 15, longitude: 0 }]);
    // The noon irradiance cube resolved to nothing, so it is left out.
    expect(Object.keys(weather?.textures ?? {})).toEqual(["sky\\sky_night", "sky\\sky_night#small", "sky\\sky_noon"]);
    expect(weather?.textures["sky\\sky_noon"]?.encoding).toBe(ERendererTextureEncoding.FETCH);
    expect(service.failure).toBeNull();
  });

  it("lights by hand, saying why, where the level offers no cycle", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription({ offered: [] })),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.weather).toBeNull();
    expect(service.failure).toBe("The level's weathers resolve to no cycle the game has");
  });

  it("reads a level once, and forgets it when it closes", async () => {
    let reads: number = 0;

    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(() => {
        reads += 1;

        return mockLevelWeatherDescription();
      }),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);
    await service.open(SELECTED);

    expect(reads).toBe(1);

    await service.open(null);

    expect(service.weather).toBeNull();
    expect(service.description).toBeNull();
  });

  it("plays nothing while the source is manual, and keeps the cycle for when it is not", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);
    service.setSource(ELevelWeatherSource.MANUAL);

    expect(service.weather).toBeNull();

    service.setSource(ELevelWeatherSource.WEATHER);

    expect(service.weather).not.toBeNull();
  });

  it("hears the clock from the renderer, and seeks anew every time", () => {
    const service: LevelWeatherService = createService();

    service.noteReport({ effect: null, keyframes: [0, 1], modifiers: 0, time: 50_000, weight: 0.5 });

    expect(service.time).toBe(50_000);

    service.seekTo(3_600);

    const seek = service.seek;

    service.seekTo(3_600);

    expect(service.time).toBe(3_600);
    expect(service.seek).not.toBe(seek);
  });

  it("plays another cycle of the game, read for the level, and remembers it with the clock per level", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);
    await service.selectCycle("default_rain");
    service.setFactor(5_000);
    service.setPlaying(true);
    service.seekTo(7_200);

    expect(service.cycle?.name).toBe("default_rain");
    expect(service.control).toEqual({ factor: 1000, isDynamicSun: false, isPaused: false });
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value))).toEqual({
      control: { factor: 1000, isDynamicSun: false, isPaused: false },
      cycle: "default_rain",
      source: ELevelWeatherSource.WEATHER,
      time: 7_200,
    });

    // The clock heard back is remembered as the level closes.
    service.noteReport({ effect: null, keyframes: [0, 1], modifiers: 0, time: 9_000, weight: 0.25 });
    await service.open(null);

    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value))?.time).toBe(9_000);
    expect(service.control.isPaused).toBe(true);

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("default_rain");
    expect(service.time).toBe(9_000);
    expect(service.control.factor).toBe(1000);
  });

  it("plays the level's own cycle where the remembered one no longer reads", async () => {
    writeLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value), {
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      cycle: "gone",
      source: ELevelWeatherSource.MANUAL,
      time: 600,
    });
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: () => Promise.reject(new Error("There is no weather cycle 'gone'")),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("default_clear");
    expect(service.failure).toBeNull();
    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    expect(service.weather).toBeNull();
    expect(service.lightingLock).toBeNull();
    expect(service.time).toBe(600);
  });

  it("says the toolbar's lighting does nothing while the weather lights the level", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createService();

    expect(service.lightingLock).toBeNull();

    await service.open(SELECTED);

    expect(service.lightingLock).not.toBeNull();
  });
});
