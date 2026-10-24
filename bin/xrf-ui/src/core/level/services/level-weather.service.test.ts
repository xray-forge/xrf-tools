import { afterEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container } from "@wirestate/core";
import { autorun } from "@wirestate/mobx";

import { SessionSnapshot } from "@/core/ipc/types/xrf-app-core";
import { SelectedLevelDescription } from "@/core/ipc/types/xrf-app-levels";
import { EXrayEngine, EXrayEngineEvidence, XrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { EWeatherCycleKind, WeatherCycleId } from "@/core/ipc/types/xrf-environment";
import { EWorldWeatherPlay, EWorldWeatherTransition } from "@/core/ipc/types/xrf-world";
import { DEFAULT_LEVEL_MANUAL_WEATHER } from "@/core/level/lib/weather/level-manual-weather";
import { DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS } from "@/core/level/lib/weather/level-sun-shafts-options";
import { ILevelWeatherHold } from "@/core/level/lib/weather/level-weather-hold";
import {
  readLevelWeatherMemory,
  toLevelWeatherMemoryKey,
  writeLevelWeatherMemory,
} from "@/core/level/lib/weather/level-weather-memory";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { clearLocalStorage } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import {
  mockLevelWeatherCycle,
  mockLevelWeatherDescription,
  mockWorldWeatherReport,
} from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { createLevelWeatherService, LEVEL_WEATHER_SELECTED, toWeatherKeyframe } from "@/fixtures/utils/level-weather";
afterEach(() => {
  resetMockInvoke();
  clearLocalStorage();
});

describe("LevelWeatherService", () => {
  it("plays the first cycle the level offers, by name", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(service.cycle.value?.name).toBe("default_clear");
    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "default_clear" });
    expect(service.transition).toBe(EWorldWeatherTransition.CUT);
    expect(service.failure).toBeNull();
  });

  it("plays the keyframe set by hand, without failing, where neither the level nor its game offers a cycle", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription({ offered: [] })),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(service.isManual).toBe(true);
    expect(toWeatherKeyframe(service.weather)?.skyTexture).toBe(DEFAULT_LEVEL_MANUAL_WEATHER.skyTexture);
    expect(service.transition).toBe(EWorldWeatherTransition.CUT);
    expect(service.failure).toBeNull();
  });

  // XRF's levels write `weathers = dynamic`, which its weather manager turns into a graph by the level's periods; any
  // fork's script-chosen weather looks the same, and the game's own cycles stand in for it.
  it("plays the game's first cycle where the level's weathers leads to none, saying so", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(
        mockLevelWeatherDescription({
          cycles: [
            {
              file: "environment\\weather_effects\\fx_blowout.ltx",
              findings: 0,
              keyframes: 4,
              kind: EWeatherCycleKind.EFFECT,
              name: "fx_blowout",
            },
            {
              file: "environment\\weathers\\w_clear.ltx",
              findings: 0,
              keyframes: 24,
              kind: EWeatherCycleKind.CYCLE,
              name: "w_clear",
            },
          ],
          offered: [],
          weather: { key: "dynamic", level: "zaton", options: [{ cycle: "dynamic", graph: null, state: null }] },
        })
      ),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(service.cycle.value?.name).toBe("w_clear");
    expect(service.isManual).toBe(false);
    expect(service.unfollowed).toBe("dynamic");
  });

  it("plays the remembered cycle where the level offers no cycle", async () => {
    writeLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value), {
      base: null,
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      cycle: "w_clear",
      manual: null,
      seed: null,
      source: ELevelWeatherSource.WEATHER,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 600,
    });
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription({ offered: [] })),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(service.cycle.value?.name).toBe("w_clear");
    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "w_clear" });
    expect(service.isManual).toBe(false);
    expect(service.failure).toBeNull();
  });

  it("plays the keyframe set by hand where the level's weather does not read", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: () => Promise.reject(new Error("No weather")),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(toWeatherKeyframe(service.weather)).not.toBeNull();
    expect(service.failure).toBe("No weather");
  });

  it("reads a level once, and forgets it when it closes", async () => {
    let reads: number = 0;

    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(() => {
        reads += 1;

        return mockLevelWeatherDescription();
      }),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);
    await service.open(LEVEL_WEATHER_SELECTED);

    expect(reads).toBe(1);

    await service.open(null);

    expect(service.weather).toBeNull();
    expect(service.description.isIdle).toBe(true);
    expect(service.description.value).toBeNull();
  });

  it("plays another cycle of the game, read for the level, and remembers it with the clock per level", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);
    await service.selectCycle("default_rain");
    service.setFactor(5_000);
    service.setPlaying(true);
    service.seekTo(7_200);

    expect(service.cycle.value?.name).toBe("default_rain");
    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "default_rain" });
    expect(service.transition).toBe(EWorldWeatherTransition.FADE);
    expect(service.control).toEqual({ factor: 1000, isDynamicSun: false, isPaused: false });
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value))).toEqual({
      base: null,
      control: { factor: 1000, isDynamicSun: false, isPaused: false },
      cycle: "default_rain",
      manual: null,
      seed: null,
      source: ELevelWeatherSource.WEATHER,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 7_200,
    });

    // The clock heard back is remembered as the level closes.
    service.noteReport(mockWorldWeatherReport({ time: 9_000, weight: 0.25 }));
    await service.open(null);

    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value))?.time).toBe(9_000);
    expect(service.control.isPaused).toBe(true);

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(service.cycle.value?.name).toBe("default_rain");
    expect(service.time).toBe(9_000);
    expect(service.control.factor).toBe(1000);
  });

  it("holds the clock, cuts to a borrowed cycle not remembered, and back to what it held", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createLevelWeatherService();
    const key: string = toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value);

    await service.open(LEVEL_WEATHER_SELECTED);
    await service.selectCycle("default_rain");
    service.setPlaying(true);

    const held: ILevelWeatherHold = service.hold();

    expect(service.control.isPaused).toBe(true);
    expect(readLevelWeatherMemory(key)?.control.isPaused).toBe(false);

    await service.cutTo("map");

    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "map" });
    expect(service.transition).toBe(EWorldWeatherTransition.CUT);
    expect(service.control.isPaused).toBe(true);
    expect(readLevelWeatherMemory(key)?.cycle).toBe("default_rain");

    await service.release(held);

    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "default_rain" });
    expect(service.transition).toBe(EWorldWeatherTransition.CUT);
    expect(service.control.isPaused).toBe(false);
    expect(readLevelWeatherMemory(key)?.control.isPaused).toBe(false);
  });

  // A floor set for one level is that level's: the next opens with its own, or with none.
  it("remembers the sun shafts with the level's weather, and opens another level without them", async () => {
    const other: SessionSnapshot<SelectedLevelDescription> = {
      sessionId: "other",
      revision: 0,
      value: mockSelectedLevelDescription({ source: { kind: "directory", path: "C:/levels/jupiter" } }),
    };

    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createLevelWeatherService();
    const shafts = { minimum: 0.3, steps: 15 };

    await service.open(LEVEL_WEATHER_SELECTED);
    service.setSunShafts(shafts);

    // Remembered as it is set, so a window closed straight after keeps it.
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value))?.sunShafts).toEqual(shafts);

    await service.open(other);

    expect(service.sunShafts).toEqual(DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS);

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(service.sunShafts).toEqual(shafts);
  });

  // Picking the held cycle again while another is read is the last word: the read landing later plays nothing.
  it("keeps the held cycle picked again while another one is read", async () => {
    let release!: () => void;

    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(
        ({ cycle }: { cycle: WeatherCycleId }) =>
          new Promise((resolve) => {
            release = () => resolve(mockLevelWeatherCycle({ name: cycle.name }));
          })
      ),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    const rain: Promise<void> = service.selectCycle("default_rain");

    expect(service.reading).toBe("default_rain");

    await service.selectCycle("default_clear");

    expect(service.reading).toBeNull();

    release();
    await rain;

    expect(service.cycle.value?.name).toBe("default_clear");
    expect(service.weather).toEqual({ kind: EWorldWeatherPlay.CYCLE, name: "default_clear" });
  });

  it("leaves a level opened during a cycle's read lit as it was", async () => {
    const other: SessionSnapshot<SelectedLevelDescription> = {
      sessionId: "other",
      revision: 0,
      value: mockSelectedLevelDescription({ source: { kind: "directory", path: "C:/levels/jupiter" } }),
    };
    let release!: () => void;

    writeLevelWeatherMemory(toLevelWeatherMemoryKey(other.value), {
      base: null,
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      cycle: "",
      manual: { ...DEFAULT_LEVEL_MANUAL_WEATHER, rainDensity: 0.3 },
      seed: null,
      source: ELevelWeatherSource.MANUAL,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 600,
    });
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(
        ({ cycle }: { cycle: WeatherCycleId }) =>
          new Promise((resolve) => {
            release = () => resolve(mockLevelWeatherCycle({ name: cycle.name }));
          })
      ),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    const rain: Promise<void> = service.selectCycle("default_rain");

    await service.open(other);
    release();
    await rain;

    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(other.value))?.source).toBe(ELevelWeatherSource.MANUAL);
    expect(toWeatherKeyframe(service.weather)?.rainDensity).toBe(0.3);
  });

  // Read while rendering, so a level opened since must reach whatever shows it.
  it("names the engine its weather is read for as each level opens", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createLevelWeatherService();
    const engines: Array<XrayEngine> = [];
    const stop = autorun(() => {
      engines.push(service.engine);
    });

    await service.open({
      sessionId: "extended",
      revision: 0,
      value: mockSelectedLevelDescription({
        engine: { engine: EXrayEngine.EXTENDED, evidence: EXrayEngineEvidence.NAMED, subject: null },
      }),
    });
    stop();

    expect(engines).toEqual([EXrayEngine.VANILLA, EXrayEngine.EXTENDED]);
  });

  it("plays the level's own cycle where the remembered one no longer reads", async () => {
    writeLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value), {
      base: null,
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      cycle: "gone",
      manual: { ...DEFAULT_LEVEL_MANUAL_WEATHER, rainDensity: 0.3 },
      seed: null,
      source: ELevelWeatherSource.MANUAL,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 600,
    });
    setMockInvokeResponses({
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: () => Promise.reject(new Error("There is no weather 'gone'")),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    expect(service.cycle.value?.name).toBe("default_clear");
    expect(service.failure).toBeNull();
    // The remembered cycle that did not read is no longer said to be reading.
    expect(service.reading).toBeNull();
    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    // The remembered keyframe set by hand lights it, cut in.
    expect(toWeatherKeyframe(service.weather)?.rainDensity).toBe(0.3);
    expect(service.transition).toBe(EWorldWeatherTransition.CUT);
    expect(service.time).toBe(600);
  });

  it("opens the weather of a level as the level opens, with no viewport to play it, and forgets it as it closes", async () => {
    setMockInvokeResponses({
      ["plugin:levels|restore_level"]: mockSessionResponse(mockSelectedLevelDescription()),
      ["plugin:levels|describe_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const container: Container = mockContainer([LevelLoadService, LevelWeatherService]).provision();
    const service: LevelWeatherService = container.get(LevelWeatherService);

    await waitFor(() => expect(service.cycle.value?.name).toBe("default_clear"));

    container.get(LevelLoadService).clear();

    expect(service.cycle.value).toBeNull();

    container.deprovision();
  });
});
