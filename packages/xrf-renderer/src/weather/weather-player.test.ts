/// <reference types="node" />

import { readFileSync } from "node:fs";
import { join } from "node:path";

import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherControl } from "#/contract/weather/renderer-weather-control";
import { ERendererWeatherEngine } from "#/contract/weather/renderer-weather-engine";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { RendererTextures } from "#/texture/renderer-textures";
import { WeatherPlayer } from "#/weather/weather-player";

const KEYFRAMES: ReadonlyArray<IRendererWeatherKeyframe> = (
  JSON.parse(readFileSync(join(__dirname, "weather-mix.golden.json"), "utf8")) as Array<{
    keyframes: Array<IRendererWeatherKeyframe>;
  }>
)[0].keyframes;

const SOURCE = {
  encoding: ERendererTextureEncoding.FETCH,
  file: { body: "", headers: {}, url: "file" },
  picture: { body: "", headers: {}, url: "picture" },
} as const;

const WEATHER: IRendererWeather = {
  engine: ERendererWeatherEngine.VANILLA,
  keyframes: KEYFRAMES,
  sunTable: null,
  textures: Object.fromEntries(
    KEYFRAMES.flatMap((keyframe: IRendererWeatherKeyframe) => [keyframe.skyTexture, keyframe.skyTextureEnv]).map(
      (reference: string) => [reference, SOURCE]
    )
  ),
};

const PLAYING: IRendererWeatherControl = {
  factor: 60,
  isDynamicSun: false,
  isFogged: true,
  isPaused: false,
  isWindy: true,
  time: 6 * 3600,
};

function createPlayer(): { player: WeatherPlayer; held: Set<string> } {
  const textures: RendererTextures = new RendererTextures(
    () => {},
    () => {}
  );
  const held: Set<string> = new Set();

  jest.spyOn(textures, "put").mockImplementation((key: string) => void held.add(key));
  jest.spyOn(textures, "release").mockImplementation((key: string) => void held.delete(key));

  return { held, player: new WeatherPlayer(textures) };
}

describe("WeatherPlayer", () => {
  it("lights nothing while no weather plays", () => {
    const { player } = createPlayer();

    expect(player.advance(0)).toBeNull();
    expect(player.isPlaying).toBe(false);
    expect(player.report).toBeNull();
  });

  it("runs the clock by the time factor, and stops it while paused", () => {
    const { player } = createPlayer();

    player.take(WEATHER);
    player.setControl(PLAYING);
    player.advance(1_000);
    player.advance(1_500);

    expect(player.report?.time).toBeCloseTo(6 * 3600 + 30, 6);

    player.setControl({ ...PLAYING, isPaused: true, time: null });

    expect(player.advance(2_000)).not.toBeNull();
    expect(player.advance(3_000)).toBeNull();
    expect(player.report?.time).toBeCloseTo(6 * 3600 + 30, 6);
  });

  it("holds the skies of the keyframes mixed and of the next, and lets the rest go", () => {
    const { player, held } = createPlayer();

    player.take(WEATHER);
    player.setControl({ ...PLAYING, isPaused: true, time: 7 * 3600 });
    player.advance(0);

    // Between six and noon, with nine at night next.
    expect([...held].sort()).toEqual(
      [KEYFRAMES[1], KEYFRAMES[2], KEYFRAMES[3]]
        .flatMap((keyframe: IRendererWeatherKeyframe) => [keyframe.skyTexture, keyframe.skyTextureEnv])
        .map((reference: string) => `@weather/${reference}`)
        .sort()
    );

    player.take(null);

    expect(held.size).toBe(0);
  });

  it("lights the scene in renderer space, fogged and swaying as the view asks", () => {
    const { player } = createPlayer();

    player.take(WEATHER);
    player.setControl({ ...PLAYING, isPaused: true, time: 12 * 3600 });

    const lit: Nullable<IRendererLighting> = player.advance(0);
    const noon: IRendererWeatherKeyframe = KEYFRAMES[2];
    const [x, y, z] = noon.sunDirection ?? [0, 0, 0];

    expect(lit?.sunDirection[0]).toBeCloseTo(x, 5);
    expect(lit?.sunDirection[1]).toBeCloseTo(y, 5);
    expect(lit?.sunDirection[2]).toBeCloseTo(-z, 5);
    expect(lit?.sky.textures).toEqual([`@weather/${KEYFRAMES[1].skyTexture}`, `@weather/${noon.skyTexture}`]);
    expect(lit?.sky.blend).toBe(1);
    expect(lit?.fog).toEqual({
      color: noon.fogColor,
      density: noon.fogDensity,
      distance: noon.fogDistance,
      farPlane: noon.farPlane,
    });

    player.setControl({ ...PLAYING, isFogged: false, isPaused: true, isWindy: false, time: null });

    const plain: Nullable<IRendererLighting> = player.advance(0);

    expect(plain?.fog).toBeNull();
    expect(plain?.trees).toBeNull();
    expect(plain?.grass).toBeNull();
  });

  it("dims the dynamic sun below the horizon", () => {
    const { player } = createPlayer();

    player.take(WEATHER);
    player.setControl({ ...PLAYING, isDynamicSun: true, isPaused: true, time: 0 });

    expect(player.advance(0)?.sunColor).toEqual([0, 0, 0]);
  });
});
