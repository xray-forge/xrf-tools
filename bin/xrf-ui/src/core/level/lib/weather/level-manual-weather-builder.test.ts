import { describe, expect, it } from "@jest/globals";
import { IRendererWeather } from "@xrf/renderer";

import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { DEFAULT_LEVEL_MANUAL_WEATHER } from "@/core/level/lib/weather/level-manual-weather";
import { LevelManualWeatherBuilder } from "@/core/level/lib/weather/level-manual-weather-builder";
import { toLevelRendererWeatherBase } from "@/core/level/lib/weather/level-renderer-weather";
import { TLevelRendererWeatherBase } from "@/core/level/lib/weather/level-renderer-weather-base";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import {
  mockLevelWeatherCycle,
  mockLevelWeatherDescription,
  mockWeatherDescriptor,
} from "@/fixtures/mocks/weather.mocks";

const ROOTS = mockSelectedLevelDescription().roots;

async function createBuilder(): Promise<{ base: TLevelRendererWeatherBase; builder: LevelManualWeatherBuilder }> {
  const base: TLevelRendererWeatherBase = await toLevelRendererWeatherBase({
    description: mockLevelWeatherDescription({
      // An effect standing no sun of its own, as an extended one does.
      effects: [
        mockLevelWeatherCycle({ keyframes: [mockWeatherDescriptor({ sunDirection: null, time: 0 })], name: "fx" }),
      ],
    }),
    roots: ROOTS,
  });

  return { base, builder: new LevelManualWeatherBuilder({ base, engine: EXrayEngine.VANILLA, roots: ROOTS }) };
}

describe("LevelManualWeatherBuilder", () => {
  // An edit is sent to the renderer as the parts that changed: one that changes no part hands over the same objects.
  it("hands over every part an edit leaves as it was as the same object", async () => {
    const { base, builder } = await createBuilder();
    const first: IRendererWeather = await builder.build(DEFAULT_LEVEL_MANUAL_WEATHER, []);
    const second: IRendererWeather = await builder.build({ ...DEFAULT_LEVEL_MANUAL_WEATHER, fogDensity: 0.1 }, []);

    expect(second.thunder).toBe(base.thunder);
    expect(second.rain).toBe(base.rain);
    expect(second.effects).toBe(first.effects);
    expect(second.textures).toBe(first.textures);
    expect(second.keyframes).not.toBe(first.keyframes);
    expect(second.sunTable).toBeNull();
  });

  it("stands an effect with no sun where the keyframe does, built again once the sun moves", async () => {
    const { builder } = await createBuilder();
    const first: IRendererWeather = await builder.build(DEFAULT_LEVEL_MANUAL_WEATHER, []);
    const moved: IRendererWeather = await builder.build({ ...DEFAULT_LEVEL_MANUAL_WEATHER, sunAltitude: 10 }, []);

    expect(first.effects.fx?.[0]?.sunDirection).toEqual(first.keyframes[0]?.sunDirection);
    expect(moved.effects).not.toBe(first.effects);
    expect(moved.effects.fx?.[0]?.sunDirection).toEqual(moved.keyframes[0]?.sunDirection);
  });

  it("asks where the keyframe's textures are fetched from again only once they change", async () => {
    const { base, builder } = await createBuilder();
    const located = [{ logicalPath: "textures\\sky\\sky_red.dds", reference: "sky\\sky_red" }];
    const first: IRendererWeather = await builder.build(DEFAULT_LEVEL_MANUAL_WEATHER, []);
    const red: IRendererWeather = await builder.build(DEFAULT_LEVEL_MANUAL_WEATHER, located);

    expect(red.textures).not.toBe(first.textures);
    expect(Object.keys(red.textures)).toEqual([...Object.keys(base.textures), "sky\\sky_red"]);
  });

  it("plays over nothing of a level whose weather does not read", async () => {
    const builder: LevelManualWeatherBuilder = new LevelManualWeatherBuilder({
      base: null,
      engine: EXrayEngine.EXTENDED,
      roots: ROOTS,
    });
    const weather: IRendererWeather = await builder.build(DEFAULT_LEVEL_MANUAL_WEATHER, []);

    expect(weather).toMatchObject({ effects: {}, modifiers: [], rain: null, textures: {}, thunder: null, wet: null });
    expect(weather.engine).toBe("extended");
  });
});
