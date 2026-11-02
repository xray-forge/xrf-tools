import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockInvoke, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { EWorldViewportEvent, EWorldWeatherPlay, EWorldWeatherTransition } from "@xrf/ipc/types/xrf-world";

import { emitViewportEvent, LEVEL_VIEWPORT, mockAttachedLevelRender } from "@/core/level/fixtures/level-render";
import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { mockWorldWeatherReport } from "@/core/level/fixtures/weather.mocks";
import { DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS } from "@/core/level/lib/weather/level-sun-shafts-options";
import { DEFAULT_LEVEL_WEATHER_CONTROL } from "@/core/level/lib/weather/level-weather-control";
import { toLevelWeatherMemoryKey, writeLevelWeatherMemory } from "@/core/level/lib/weather/level-weather-memory";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelWeatherService } from "@/core/level/services/level-weather";
import { clearLocalStorage } from "@/core/storage";
describe("LevelRenderWeather", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  // The renderer's clock starts at noon, so the time a level was left at reaches it with the level's weather.
  it("plays a level's weather on from the time it was last left at", async () => {
    writeLevelWeatherMemory(toLevelWeatherMemoryKey(mockSelectedLevelDescription()), {
      base: null,
      control: DEFAULT_LEVEL_WEATHER_CONTROL,
      cycle: "",
      manual: null,
      seed: null,
      source: ELevelWeatherSource.WEATHER,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 7_200,
    });

    await mockAttachedLevelRender();

    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.SEEK_WEATHER)).toEqual([{ time: 7_200, viewport: LEVEL_VIEWPORT }])
    );
    expect(listMockInvokeCalls(ERenderCommand.PLAY_WEATHER).at(-1)?.play).toEqual({
      kind: EWorldWeatherPlay.CYCLE,
      name: "default_clear",
    });
  });

  it("plays the level's weather in its viewport, runs its clock as asked, and hears where it stands", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const weather: LevelWeatherService = container.get(LevelWeatherService);

    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.PLAY_WEATHER).at(-1)).toEqual({
        play: { kind: EWorldWeatherPlay.CYCLE, name: "default_clear" },
        transition: EWorldWeatherTransition.CUT,
        viewport: LEVEL_VIEWPORT,
      })
    );
    expect(listMockInvokeCalls(ERenderCommand.SET_WEATHER_CONTROL).at(-1)).toEqual({
      control: { factor: 12, isDynamicSun: false, isPaused: true },
      viewport: LEVEL_VIEWPORT,
    });

    mockInvoke.mockClear();
    weather.setPlaying(true);
    service.weather.seek(3_600);
    // The same time again seeks again: the clock has run on since.
    service.weather.seek(3_600);
    service.weather.playEffect("fx_storm");
    service.weather.playAmbientEffect();
    service.weather.playAmbientEffect();

    await waitFor(() => expect(listMockInvokeCalls(ERenderCommand.PLAY_AMBIENT_EFFECT)).toHaveLength(2));
    expect(listMockInvokeCalls(ERenderCommand.SET_WEATHER_CONTROL)).toEqual([
      { control: { factor: 12, isDynamicSun: false, isPaused: false }, viewport: LEVEL_VIEWPORT },
    ]);
    expect(listMockInvokeCalls(ERenderCommand.SEEK_WEATHER)).toEqual([
      { time: 3_600, viewport: LEVEL_VIEWPORT },
      { time: 3_600, viewport: LEVEL_VIEWPORT },
    ]);
    expect(weather.time).toBe(3_600);
    expect(listMockInvokeCalls(ERenderCommand.PLAY_WEATHER_EFFECT)).toEqual([
      { name: "fx_storm", viewport: LEVEL_VIEWPORT },
    ]);
    expect(listMockInvokeCalls(ERenderCommand.PLAY_AMBIENT_EFFECT)).toEqual([
      { viewport: LEVEL_VIEWPORT },
      { viewport: LEVEL_VIEWPORT },
    ]);

    emitViewportEvent({ kind: EWorldViewportEvent.WEATHER, report: mockWorldWeatherReport({ time: 4_000 }) });
    expect(weather.time).toBe(4_000);
    expect(weather.report?.between).toEqual([0, 43_200]);
  });
});
