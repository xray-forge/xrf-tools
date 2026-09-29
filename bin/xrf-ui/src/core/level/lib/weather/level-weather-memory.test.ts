import { afterEach, describe, expect, it } from "@jest/globals";

import {
  ILevelWeatherMemory,
  readLevelWeatherMemory,
  toLevelWeatherMemory,
  toLevelWeatherMemoryKey,
  writeLevelWeatherMemory,
} from "@/core/level/lib/weather/level-weather-memory";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LEVEL_WEATHER_STORAGE_KEY } from "@/core/storage";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";

const MEMORY: ILevelWeatherMemory = {
  control: { factor: 60, isDynamicSun: true, isPaused: false },
  cycle: "default_rain",
  source: ELevelWeatherSource.WEATHER,
  time: 3_600,
};

afterEach(() => {
  window.localStorage.clear();
});

describe("level weather memory", () => {
  it("reads back what was written, per level", () => {
    writeLevelWeatherMemory("zaton", MEMORY);
    writeLevelWeatherMemory("jupiter", { ...MEMORY, cycle: "default_clear" });

    expect(readLevelWeatherMemory("zaton")).toEqual(MEMORY);
    expect(readLevelWeatherMemory("jupiter")?.cycle).toBe("default_clear");
    expect(readLevelWeatherMemory("pripyat")).toBeNull();
  });

  it("forgets the least recently played level past thirty-two", () => {
    for (let index: number = 0; index < 33; index += 1) {
      writeLevelWeatherMemory(`level-${index}`, MEMORY);
    }

    expect(readLevelWeatherMemory("level-0")).toBeNull();
    expect(readLevelWeatherMemory("level-32")).toEqual(MEMORY);

    // Played again, the oldest becomes the newest.
    writeLevelWeatherMemory("level-1", MEMORY);
    writeLevelWeatherMemory("level-33", MEMORY);

    expect(readLevelWeatherMemory("level-1")).toEqual(MEMORY);
    expect(readLevelWeatherMemory("level-2")).toBeNull();
  });

  it("reads nothing out of what does not read as a memory, and keeps what it reads within bounds", () => {
    window.localStorage.setItem(LEVEL_WEATHER_STORAGE_KEY, "not json");

    expect(readLevelWeatherMemory("zaton")).toBeNull();
    expect(toLevelWeatherMemory({ ...MEMORY, source: "sky" })).toBeNull();
    expect(toLevelWeatherMemory({ ...MEMORY, time: "noon" })).toBeNull();
    expect(toLevelWeatherMemory({ ...MEMORY, control: { factor: 5_000 }, time: 100_000 })).toEqual({
      ...MEMORY,
      control: { factor: 1000, isDynamicSun: false, isPaused: true },
      time: 86_399,
    });
  });

  it("keys a level by the roots it opened in, so two games' levels of one name are apart", () => {
    const level = mockSelectedLevelDescription();

    expect(toLevelWeatherMemoryKey(level)).not.toBe(
      toLevelWeatherMemoryKey({ ...level, roots: { ...level.roots, roots: [{ mode: "auto", path: "D:\\anomaly" }] } })
    );
  });
});
