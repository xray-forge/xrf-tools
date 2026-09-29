import { afterEach, describe, expect, it } from "@jest/globals";
import { ERendererEngine, ERendererTextureEncoding, ERendererWeatherTransition, IRendererWeather } from "@xrf/renderer";
import { Nullable, Optional } from "@xrf/types";

import { LevelTextureReference, SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { WeatherCycleId } from "@/core/ipc/types/xrf-environment";
import {
  DEFAULT_LEVEL_MANUAL_WEATHER,
  listLevelManualWeatherTextures,
  toLevelManualWeather,
} from "@/core/level/lib/weather/level-manual-weather";
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
import {
  mockLevelWeatherCycle,
  mockLevelWeatherDescription,
  mockRendererWeatherReport,
} from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";

const SELECTED: SessionSnapshot<SelectedLevelDescription> = {
  sessionId: "level",
  value: mockSelectedLevelDescription(),
};

/** Every reference set by hand resolves beside the game's own textures. */
const RESOLVE = {
  ["plugin:levels|resolve_level_textures"]: mockSessionResponse(({ references }: { references: Array<string> }) =>
    toResolved(references)
  ),
};

function toResolved(references: Array<string>): Array<LevelTextureReference> {
  return references.map((reference: string) => ({ logicalPath: `textures/${reference}.dds`, reference }));
}

/** Lets the builds of the keyframe set by hand, which ask the backend, settle. */
async function settle(): Promise<void> {
  for (let index: number = 0; index < 10; index += 1) {
    await Promise.resolve();
  }
}

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
    expect(weather?.engine).toBe(ERendererEngine.EXTENDED);
    expect(weather?.keyframes.map((keyframe) => keyframe.time)).toEqual([0, LEVEL_WEATHER_NOON]);
    expect(weather?.sunTable).toEqual([{ altitude: 15, longitude: 0 }]);
    // The noon irradiance cube resolved to nothing, so it is left out.
    expect(Object.keys(weather?.textures ?? {})).toEqual([
      "sky\\sky_night",
      "sky\\sky_night#small",
      "sky\\sky_noon",
      "fx\\fx_rain",
      "water\\water_SBumpVolume",
      "water\\water_flowing_nmap",
    ]);
    expect(weather?.textures["sky\\sky_noon"]?.encoding).toBe(ERendererTextureEncoding.FETCH);
    expect(service.failure).toBeNull();
  });

  it("plays the keyframe set by hand, saying why, where the level offers no cycle", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription({ offered: [] })),
      ...RESOLVE,
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.isManual).toBe(true);
    expect(service.weather?.keyframes).toHaveLength(1);
    expect(service.weather?.keyframes[0]?.skyTexture).toBe(DEFAULT_LEVEL_MANUAL_WEATHER.skyTexture);
    expect(service.weather?.sunTable).toBeNull();
    expect(service.transition).toBe(ERendererWeatherTransition.CUT);
    expect(service.failure).toBe("The level's weathers resolve to no cycle the game has");
  });

  it("plays the keyframe set by hand for the engine target where the level's weather does not read", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: () => Promise.reject(new Error("No weather")),
      ...RESOLVE,
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.weather?.keyframes).toHaveLength(1);
    expect(service.weather?.effects).toEqual({});
    expect(service.failure).toBe("No weather");
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

  it("fades into the keyframe set by hand, seeded from what is shown, and back into the cycle kept", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ...RESOLVE,
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    const cycle: Nullable<IRendererWeather> = service.weather;

    expect(service.transition).toBe(ERendererWeatherTransition.CUT);

    service.noteReport(mockRendererWeatherReport({ time: 50_000 }));
    service.setSource(ELevelWeatherSource.MANUAL);
    await settle();

    expect(service.manual).toEqual(toLevelManualWeather(mockRendererWeatherReport({ time: 50_000 }).current));
    expect(service.seed).toEqual({ cycle: "default_clear", time: 50_000 });
    expect(service.weather?.keyframes).toHaveLength(1);
    expect(service.transition).toBe(ERendererWeatherTransition.FADE);

    service.setSource(ELevelWeatherSource.WEATHER);

    expect(service.weather).toBe(cycle);
    expect(service.transition).toBe(ERendererWeatherTransition.FADE);
  });

  it("takes manual control at the first edit while the weather plays, seeded so only the edit is seen", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ...RESOLVE,
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);
    service.noteReport(
      mockRendererWeatherReport({
        current: { ...mockRendererWeatherReport().current, skyTexture: "sky\\sky_night" },
      })
    );
    service.editManual({ rainDensity: 0.5 });
    await settle();

    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    expect(service.manual?.skyTexture).toBe("sky\\sky_night");
    expect(service.manual?.rainDensity).toBe(0.5);
    expect(service.weather?.keyframes[0]?.rainDensity).toBe(0.5);
    expect(service.weather?.textures["sky\\sky_night"]).toBeDefined();

    service.editManual({ skyTexture: "sky\\sky_noon" });
    await settle();

    expect(service.weather?.keyframes[0]?.skyTexture).toBe("sky\\sky_noon");
    expect(service.transition).toBe(ERendererWeatherTransition.EASE);
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value))?.manual?.skyTexture).toBe("sky\\sky_noon");
  });

  it("asks where a texture set by hand resolves once", async () => {
    const asked: Array<Array<string>> = [];

    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|resolve_level_textures"]: mockSessionResponse(({ references }: { references: Array<string> }) => {
        asked.push(references);

        return toResolved(references);
      }),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);
    service.editManual({ rainDensity: 0.5 });
    await settle();
    service.editManual({ rainDensity: 0.6 });
    await settle();

    expect(asked).toEqual([listLevelManualWeatherTextures(DEFAULT_LEVEL_MANUAL_WEATHER)]);
  });

  it("keeps where a texture set by hand resolves to the level it was asked for", async () => {
    const gate: { release: Optional<() => void> } = { release: undefined };
    const held: Promise<void> = new Promise((resolve) => {
      gate.release = resolve;
    });

    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|resolve_level_textures"]: mockSessionResponse(
        async ({ sessionId, references }: { sessionId: string; references: Array<string> }) => {
          // The first level's answer comes only once another level is open.
          if (sessionId === SELECTED.sessionId) {
            await held;
          }

          return references.map((reference: string) => ({ logicalPath: `${sessionId}/${reference}.dds`, reference }));
        }
      ),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);
    service.editManual({ rainDensity: 0.5 });
    await settle();
    await service.open({ ...SELECTED, sessionId: "other" });
    gate.release?.();
    await settle();
    service.editManual({ rainDensity: 0.6 });
    await settle();

    const fetched: string = JSON.stringify(service.manualPlayable?.textures);

    expect(fetched).toContain("other/sky");
    expect(fetched).not.toContain(`${SELECTED.sessionId}/`);
  });

  it("hears the clock from the renderer, and seeks anew every time", () => {
    const service: LevelWeatherService = createService();

    service.noteReport(mockRendererWeatherReport({ time: 50_000 }));

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
      manual: null,
      seed: null,
      source: ELevelWeatherSource.WEATHER,
      time: 7_200,
    });

    // The clock heard back is remembered as the level closes.
    service.noteReport(mockRendererWeatherReport({ time: 9_000, weight: 0.25 }));
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
      manual: { ...DEFAULT_LEVEL_MANUAL_WEATHER, rainDensity: 0.3 },
      seed: null,
      source: ELevelWeatherSource.MANUAL,
      time: 600,
    });
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: () => Promise.reject(new Error("There is no weather cycle 'gone'")),
      ...RESOLVE,
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("default_clear");
    expect(service.failure).toBeNull();
    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    // The remembered keyframe set by hand lights it, cut in.
    expect(service.weather?.keyframes[0]?.rainDensity).toBe(0.3);
    expect(service.transition).toBe(ERendererWeatherTransition.CUT);
    expect(service.time).toBe(600);
  });
});
