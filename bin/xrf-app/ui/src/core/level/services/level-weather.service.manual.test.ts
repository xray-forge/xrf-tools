import { afterEach, describe, expect, it } from "@jest/globals";
import { autorun } from "@wirestate/mobx";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { EWorldWeatherTransition, WorldWeatherPlay } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

import {
  createLevelWeatherService,
  LEVEL_WEATHER_SELECTED,
  toWeatherKeyframe,
} from "@/core/level/fixtures/level-weather";
import { mockLevelWeatherDescription, mockWorldWeatherReport } from "@/core/level/fixtures/weather.mocks";
import { ILevelManualWeather, toLevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { readLevelWeatherMemory, toLevelWeatherMemoryKey } from "@/core/level/lib/weather/level-weather-memory";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { clearLocalStorage } from "@/core/storage";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";

afterEach(() => {
  resetMockInvoke();
  clearLocalStorage();
});

describe("LevelWeatherService, set by hand", () => {
  it("fades into the keyframe set by hand, seeded from what is shown, and back into the cycle kept", async () => {
    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_WEATHER]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    const cycle: Nullable<WorldWeatherPlay> = service.weather;

    expect(service.transition).toBe(EWorldWeatherTransition.CUT);

    service.noteReport(mockWorldWeatherReport({ time: 50_000 }));
    service.setSource(ELevelWeatherSource.MANUAL);

    expect(service.manual).toEqual(toLevelManualWeather(mockWorldWeatherReport({ time: 50_000 }).current));
    expect(service.seed).toEqual({
      cycle: "default_clear",
      time: 50_000,
      weather: toLevelManualWeather(mockWorldWeatherReport({ time: 50_000 }).current),
    });
    expect(toWeatherKeyframe(service.weather)).not.toBeNull();
    expect(service.transition).toBe(EWorldWeatherTransition.FADE);

    service.setSource(ELevelWeatherSource.WEATHER);

    expect(service.weather).toEqual(cycle);
    expect(service.transition).toBe(EWorldWeatherTransition.FADE);
  });

  it("resets keys to the weather it was seeded from, and nothing while the weather plays", async () => {
    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_WEATHER]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);
    service.noteReport(
      mockWorldWeatherReport({
        current: { ...mockWorldWeatherReport().current, skyTexture: "sky\\sky_night" },
      })
    );

    service.resetManual(["skyTexture"]);

    expect(service.source).toBe(ELevelWeatherSource.WEATHER);

    service.editManual({ skyTexture: "sky\\sky_noon", rainDensity: 0.5 });
    service.resetManual(["skyTexture"]);

    expect(service.manual?.skyTexture).toBe("sky\\sky_night");
    expect(service.manual?.rainDensity).toBe(0.5);
    expect(
      readLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value))?.seed?.weather?.skyTexture
    ).toBe("sky\\sky_night");
  });

  it("takes manual control at the first edit while the weather plays, seeded so only the edit is seen", async () => {
    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_WEATHER]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);
    service.noteReport(
      mockWorldWeatherReport({
        current: {
          ...mockWorldWeatherReport().current,
          ambient: "ambient_rain",
          skyTexture: "sky\\sky_night",
          skyTextureEnv: "sky\\sky_night_env",
        },
      })
    );
    service.editManual({ rainDensity: 0.5 });

    expect(service.source).toBe(ELevelWeatherSource.MANUAL);
    expect(service.manual?.skyTexture).toBe("sky\\sky_night");
    expect(service.manual?.rainDensity).toBe(0.5);
    expect(toWeatherKeyframe(service.weather)?.rainDensity).toBe(0.5);
    expect(toWeatherKeyframe(service.weather)?.skyTextureEnv).toBe("sky\\sky_night_env");
    expect(toWeatherKeyframe(service.weather)?.ambient).toBe("ambient_rain");
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value))?.base?.ambient).toBe(
      "ambient_rain"
    );

    service.editManual({ skyTexture: "sky\\sky_noon" });

    expect(toWeatherKeyframe(service.weather)?.skyTexture).toBe("sky\\sky_noon");
    expect(service.transition).toBe(EWorldWeatherTransition.EASE);
    expect(readLevelWeatherMemory(toLevelWeatherMemoryKey(LEVEL_WEATHER_SELECTED.value))?.manual?.skyTexture).toBe(
      "sky\\sky_noon"
    );
  });

  // The renderer reports a few times a second while its clock runs, and the level's toolbar is drawn from the keyframe
  // shown.
  it("shows the same keyframe for a report the same as the last, and reads no report lit by hand", async () => {
    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_WEATHER]: mockSessionResponse(mockLevelWeatherDescription()),
    });

    const service: LevelWeatherService = createLevelWeatherService();

    await service.open(LEVEL_WEATHER_SELECTED);

    const shown: Array<ILevelManualWeather> = [];
    const stop: () => void = autorun(() => void shown.push(service.shown));

    service.noteReport(mockWorldWeatherReport());
    service.noteReport(mockWorldWeatherReport());

    expect(shown).toHaveLength(2);

    service.setSource(ELevelWeatherSource.MANUAL);

    const seen: number = shown.length;

    service.noteReport(mockWorldWeatherReport({ time: 50_000 }));
    stop();

    expect(shown).toHaveLength(seen);
  });

  it("hears the clock from the renderer, and stands where it was sent", () => {
    const service: LevelWeatherService = createLevelWeatherService();

    service.noteReport(mockWorldWeatherReport({ time: 50_000 }));

    expect(service.time).toBe(50_000);

    service.seekTo(3_600);

    expect(service.time).toBe(3_600);
  });
});
