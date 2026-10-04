import { afterEach, describe, expect, it } from "@jest/globals";
import { autorun } from "@wirestate/mobx";
import { Nullable } from "@xrf/types";

import { SelectedLevelDescription, SessionSnapshot } from "@/core/ipc/types/xrf-app";
import { EWeatherCycleKind, WeatherCycleId, WeatherDescriptor } from "@/core/ipc/types/xrf-environment";
import {
  ERenderSunShaftsQuality,
  ERenderWeatherPlay,
  ERenderWeatherTransition,
  RenderWeatherPlay,
} from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_LEVEL_MANUAL_WEATHER,
  ILevelManualWeather,
  toLevelManualWeather,
} from "@/core/level/lib/weather/level-manual-weather";
import { DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS } from "@/core/level/lib/weather/level-sun-shafts-options";
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
  mockRenderWeatherReport,
} from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";

const SELECTED: SessionSnapshot<SelectedLevelDescription> = {
  sessionId: "level",
  value: mockSelectedLevelDescription(),
};

function createService(): LevelWeatherService {
  return mockContainer([LevelWeatherService]).get(LevelWeatherService);
}

/** The keyframe set by hand a play hands over, or null for a play of anything else. */
function toKeyframe(play: Nullable<RenderWeatherPlay>): Nullable<WeatherDescriptor> {
  return play?.kind === ERenderWeatherPlay.KEYFRAME ? play.keyframe : null;
}

afterEach(() => {
  resetMockInvoke();
  window.localStorage.clear();
});

describe("LevelWeatherService", () => {
  it("plays the first cycle the level offers, by name", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("default_clear");
    expect(service.weather).toEqual({ kind: ERenderWeatherPlay.CYCLE, name: "default_clear" });
    expect(service.transition).toBe(ERenderWeatherTransition.CUT);
    expect(service.failure).toBeNull();
  });

  it("plays the keyframe set by hand, without failing, where neither the level nor its game offers a cycle", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription({ offered: [] })),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.isManual).toBe(true);
    expect(toKeyframe(service.weather)?.skyTexture).toBe(DEFAULT_LEVEL_MANUAL_WEATHER.skyTexture);
    expect(service.transition).toBe(ERenderWeatherTransition.CUT);
    expect(service.failure).toBeNull();
  });

  // XRF's levels write `weathers = dynamic`, which its weather manager turns into a graph by the level's periods; any
  // fork's script-chosen weather looks the same, and the game's own cycles stand in for it.
  it("plays the game's first cycle where the level's weathers leads to none, saying so", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(
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

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("w_clear");
    expect(service.isManual).toBe(false);
    expect(service.unfollowed).toBe("dynamic");
  });

  it("plays the remembered cycle where the level offers no cycle", async () => {
    writeLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value), {
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      cycle: "w_clear",
      manual: null,
      seed: null,
      source: ELevelWeatherSource.WEATHER,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 600,
    });
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription({ offered: [] })),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("w_clear");
    expect(service.weather).toEqual({ kind: ERenderWeatherPlay.CYCLE, name: "w_clear" });
    expect(service.isManual).toBe(false);
    expect(service.failure).toBeNull();
  });

  it("plays the keyframe set by hand where the level's weather does not read", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: () => Promise.reject(new Error("No weather")),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(toKeyframe(service.weather)).not.toBeNull();
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
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    const cycle: Nullable<RenderWeatherPlay> = service.weather;

    expect(service.transition).toBe(ERenderWeatherTransition.CUT);

    service.noteReport(mockRenderWeatherReport({ time: 50_000 }));
    service.setSource(ELevelWeatherSource.MANUAL);

    expect(service.manual).toEqual(toLevelManualWeather(mockRenderWeatherReport({ time: 50_000 }).current));
    expect(service.seed).toEqual({ cycle: "default_clear", time: 50_000 });
    expect(toKeyframe(service.weather)).not.toBeNull();
    expect(service.transition).toBe(ERenderWeatherTransition.FADE);

    service.setSource(ELevelWeatherSource.WEATHER);

    expect(service.weather).toEqual(cycle);
    expect(service.transition).toBe(ERenderWeatherTransition.FADE);
  });

  it("takes manual control at the first edit while the weather plays, seeded so only the edit is seen", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);
    service.noteReport(
      mockRenderWeatherReport({
        current: { ...mockRenderWeatherReport().current, skyTexture: "sky\\sky_night" },
      })
    );
    service.editManual({ rainDensity: 0.5 });

    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    expect(service.manual?.skyTexture).toBe("sky\\sky_night");
    expect(service.manual?.rainDensity).toBe(0.5);
    expect(toKeyframe(service.weather)?.rainDensity).toBe(0.5);
    expect(toKeyframe(service.weather)?.skyTextureEnv).toBe("sky\\sky_night#small");

    service.editManual({ skyTexture: "sky\\sky_noon" });

    expect(toKeyframe(service.weather)?.skyTexture).toBe("sky\\sky_noon");
    expect(service.transition).toBe(ERenderWeatherTransition.EASE);
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value))?.manual?.skyTexture).toBe("sky\\sky_noon");
  });

  // The renderer reports a few times a second while its clock runs, and the level's toolbar is drawn from the keyframe
  // shown.
  it("shows the same keyframe for a report the same as the last, and reads no report lit by hand", async () => {
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    const shown: Array<ILevelManualWeather> = [];
    const stop: () => void = autorun(() => void shown.push(service.shown));

    service.noteReport(mockRenderWeatherReport());
    service.noteReport(mockRenderWeatherReport());

    expect(shown).toHaveLength(2);

    service.setSource(ELevelWeatherSource.MANUAL);

    const seen: number = shown.length;

    service.noteReport(mockRenderWeatherReport({ time: 50_000 }));
    stop();

    expect(shown).toHaveLength(seen);
  });

  it("hears the clock from the renderer, and seeks anew every time", () => {
    const service: LevelWeatherService = createService();

    service.noteReport(mockRenderWeatherReport({ time: 50_000 }));

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
    expect(service.weather).toEqual({ kind: ERenderWeatherPlay.CYCLE, name: "default_rain" });
    expect(service.transition).toBe(ERenderWeatherTransition.FADE);
    expect(service.control).toEqual({ factor: 1000, isDynamicSun: false, isPaused: false });
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value))).toEqual({
      control: { factor: 1000, isDynamicSun: false, isPaused: false },
      cycle: "default_rain",
      manual: null,
      seed: null,
      source: ELevelWeatherSource.WEATHER,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 7_200,
    });

    // The clock heard back is remembered as the level closes.
    service.noteReport(mockRenderWeatherReport({ time: 9_000, weight: 0.25 }));
    await service.open(null);

    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value))?.time).toBe(9_000);
    expect(service.control.isPaused).toBe(true);

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("default_rain");
    expect(service.time).toBe(9_000);
    expect(service.control.factor).toBe(1000);
  });

  // A floor set for one level is that level's: the next opens with its own, or with none.
  it("remembers the sun shafts with the level's weather, and opens another level without them", async () => {
    const other: SessionSnapshot<SelectedLevelDescription> = {
      sessionId: "other",
      value: mockSelectedLevelDescription({ source: { kind: "directory", path: "C:/levels/jupiter" } }),
    };

    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: mockSessionResponse(({ cycle }: { cycle: WeatherCycleId }) =>
        mockLevelWeatherCycle({ name: cycle.name })
      ),
    });

    const service: LevelWeatherService = createService();
    const shafts = { minimum: 0.3, quality: ERenderSunShaftsQuality.LOW };

    await service.open(SELECTED);
    service.setSunShafts(shafts);
    await service.open(other);

    expect(service.sunShafts).toEqual(DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS);

    await service.open(SELECTED);

    expect(service.sunShafts).toEqual(shafts);
  });

  it("plays the level's own cycle where the remembered one no longer reads", async () => {
    writeLevelWeatherMemory(toLevelWeatherMemoryKey(SELECTED.value), {
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      cycle: "gone",
      manual: { ...DEFAULT_LEVEL_MANUAL_WEATHER, rainDensity: 0.3 },
      seed: null,
      source: ELevelWeatherSource.MANUAL,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 600,
    });
    setMockInvokeResponses({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|read_level_cycle"]: () => Promise.reject(new Error("There is no weather 'gone'")),
    });

    const service: LevelWeatherService = createService();

    await service.open(SELECTED);

    expect(service.cycle?.name).toBe("default_clear");
    expect(service.failure).toBeNull();
    // The remembered cycle that did not read is no longer said to be reading.
    expect(service.reading).toBeNull();
    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    // The remembered keyframe set by hand lights it, cut in.
    expect(toKeyframe(service.weather)?.rainDensity).toBe(0.3);
    expect(service.transition).toBe(ERenderWeatherTransition.CUT);
    expect(service.time).toBe(600);
  });
});
