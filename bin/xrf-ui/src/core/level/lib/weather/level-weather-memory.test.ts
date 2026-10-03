import { afterEach, describe, expect, it } from "@jest/globals";

import { ERenderSunShaftsQuality } from "@/core/ipc/types/xrf-renderer";
import { DEFAULT_LEVEL_MANUAL_WEATHER } from "@/core/level/lib/weather/level-manual-weather";
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
  manual: null,
  seed: null,
  source: ELevelWeatherSource.WEATHER,
  sunShafts: { minimum: 0.2, quality: ERenderSunShaftsQuality.MEDIUM },
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

  it("reads back the keyframe set by hand and what it was seeded from", () => {
    const memory: ILevelWeatherMemory = {
      ...MEMORY,
      manual: { ...DEFAULT_LEVEL_MANUAL_WEATHER, rainDensity: 0.4 },
      seed: { cycle: "default_rain", time: 3_600 },
      source: ELevelWeatherSource.MANUAL,
    };

    writeLevelWeatherMemory("zaton", memory);

    expect(readLevelWeatherMemory("zaton")).toEqual(memory);
  });

  it("forgets the least recently played levels past what storage is kept under", () => {
    const manual = { ...DEFAULT_LEVEL_MANUAL_WEATHER, skyTexture: "x".repeat(8_000) };

    for (let index: number = 0; index < 12; index += 1) {
      writeLevelWeatherMemory(`level-${index}`, { ...MEMORY, manual });
    }

    expect((window.localStorage.getItem(LEVEL_WEATHER_STORAGE_KEY) ?? "").length).toBeLessThanOrEqual(64 * 1024);
    expect(readLevelWeatherMemory("level-0")).toBeNull();
    expect(readLevelWeatherMemory("level-11")?.manual).toEqual(manual);
  });

  it("drops a memory of another version or with a keyframe that does not read, rather than migrating it", () => {
    const stored = { ...MEMORY, version: 3 };

    expect(toLevelWeatherMemory(stored)).toEqual(MEMORY);
    expect(toLevelWeatherMemory({ ...stored, version: 2 })).toBeNull();
    expect(toLevelWeatherMemory({ ...MEMORY })).toBeNull();
    expect(
      toLevelWeatherMemory({ ...stored, manual: { ...DEFAULT_LEVEL_MANUAL_WEATHER, sunColor: [1, 1] } })
    ).toBeNull();
    expect(toLevelWeatherMemory({ ...stored, manual: { skyTexture: "sky_7_cube" } })).toBeNull();
  });

  it("reads nothing out of what does not read as a memory, and keeps what it reads within bounds", () => {
    window.localStorage.setItem(LEVEL_WEATHER_STORAGE_KEY, "not json");

    expect(readLevelWeatherMemory("zaton")).toBeNull();
    expect(toLevelWeatherMemory({ ...MEMORY, source: "sky", version: 3 })).toBeNull();
    expect(toLevelWeatherMemory({ ...MEMORY, time: "noon", version: 3 })).toBeNull();
    expect(toLevelWeatherMemory({ ...MEMORY, control: { factor: 5_000 }, time: 100_000, version: 3 })).toEqual({
      ...MEMORY,
      control: { factor: 1000, isDynamicSun: false, isPaused: true },
      time: 86_399,
    });
    expect(toLevelWeatherMemory({ ...MEMORY, sunShafts: { minimum: 2, quality: "ultra" }, version: 3 })).toEqual({
      ...MEMORY,
      sunShafts: { minimum: 0.5, quality: ERenderSunShaftsQuality.HIGH },
    });
  });

  it("keys a level by the roots it opened in, so two games' levels of one name are apart", () => {
    const level = mockSelectedLevelDescription();

    expect(toLevelWeatherMemoryKey(level)).not.toBe(
      toLevelWeatherMemoryKey({ ...level, roots: { ...level.roots, roots: [{ mode: "auto", path: "D:\\anomaly" }] } })
    );
  });
});
