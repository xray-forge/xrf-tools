import { describe, expect, it, jest } from "@jest/globals";
import { WorldWeatherReport } from "@xrf/ipc/types/xrf-world";
import { Nullable } from "@xrf/types";

import { mockWorldWeatherReport } from "@/core/level/fixtures/weather.mocks";
import { DEFAULT_LEVEL_WEATHER_CONTROL, LEVEL_WEATHER_NOON } from "@/core/level/lib/weather/level-weather-control";
import { LevelWeatherClock } from "@/core/level/services/level-weather/level-weather.service.clock";

describe("LevelWeatherClock", () => {
  it("remembers each change asked for by hand, and nothing the renderer reports or a borrower holds", () => {
    const persist = jest.fn<() => void>();
    const clock: LevelWeatherClock = new LevelWeatherClock(persist);

    clock.noteReport(mockWorldWeatherReport({ time: 9_000 }));
    clock.hold();
    clock.release(DEFAULT_LEVEL_WEATHER_CONTROL);
    clock.restore({ control: DEFAULT_LEVEL_WEATHER_CONTROL, time: 600 });
    clock.reset();

    expect(persist).not.toHaveBeenCalled();

    clock.setPlaying(true);
    clock.setFactor(20);
    clock.setDynamicSun(true);
    clock.seekTo(7_200);

    expect(persist).toHaveBeenCalledTimes(4);
  });

  it("stands where the renderer reports, dropping a report the same as the last", () => {
    const clock: LevelWeatherClock = new LevelWeatherClock(() => undefined);

    clock.noteReport(mockWorldWeatherReport({ time: 9_000 }));

    const report: Nullable<WorldWeatherReport> = clock.report;

    clock.noteReport(mockWorldWeatherReport({ time: 9_000 }));

    expect(clock.report).toBe(report);
    expect(clock.time).toBe(9_000);

    clock.reset();

    expect(clock.report).toBeNull();
    expect(clock.time).toBe(LEVEL_WEATHER_NOON);
  });
});
