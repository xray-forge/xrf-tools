import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { IRendererLighting } from "#/contract/renderer-lighting";
import { TRendererVector } from "#/contract/renderer-vector";
import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { ERendererTextureEncoding } from "#/contract/scene/renderer-texture-source";
import { IRendererThunder } from "#/contract/weather/renderer-thunder";
import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherControl } from "#/contract/weather/renderer-weather-control";
import { ERendererWeatherEngine } from "#/contract/weather/renderer-weather-engine";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { ERendererWeatherTransition } from "#/contract/weather/renderer-weather-transition";
import { RendererTextures } from "#/texture/renderer-textures";
import { readWeatherMixGolden } from "#/weather/weather-mix-fixtures";
import { WeatherPlayer } from "#/weather/weather-player";

const KEYFRAMES: ReadonlyArray<IRendererWeatherKeyframe> = readWeatherMixGolden<
  Array<{
    keyframes: Array<IRendererWeatherKeyframe>;
  }>
>()[0].keyframes;

const SOURCE = {
  encoding: ERendererTextureEncoding.FETCH,
  file: { body: "", headers: {}, url: "file" },
  picture: { body: "", headers: {}, url: "picture" },
} as const;

/** Where the camera stands, in engine space. */
const ORIGIN: TRendererVector = [0, 0, 0];

const WEATHER: IRendererWeather = {
  effects: {},
  engine: ERendererWeatherEngine.VANILLA,
  keyframes: KEYFRAMES,
  modifiers: [],
  rain: null,
  sunTable: null,
  thunder: null,
  wet: null,
  textures: Object.fromEntries(
    KEYFRAMES.flatMap((keyframe: IRendererWeatherKeyframe) => [
      keyframe.skyTexture,
      keyframe.skyTextureEnv,
      keyframe.cloudsTexture,
    ]).map((reference: string) => [reference, SOURCE])
  ),
};

const PLAYING: IRendererWeatherControl = {
  factor: 60,
  isClouded: true,
  isDynamicSun: false,
  isFogged: true,
  isPaused: false,
  isRainy: true,
  isThundering: true,
  isWindy: true,
  time: 6 * 3600,
};

/** One bolt, lit white then black over its strike, where vanilla's `[environment]` strikes it. */
const THUNDER: IRendererThunder = {
  animators: [
    {
      colors: [
        [255, 255, 255],
        [0, 0, 0],
      ],
      fps: 2,
      frameCount: 2,
      frames: [0, 1],
    },
  ],
  bolts: {
    bolt: {
      center: { draw: ERendererDraw.ADDED, opacity: 0.6, radius: [2, 1], texture: "fx\\fx_thunderbolts_gradient" },
      color: 0,
      model: 0,
      top: { draw: ERendererDraw.ADDED, opacity: 0.6, radius: [0.5, 0.25], texture: "fx\\fx_thunderbolts_gradient" },
    },
  },
  collections: { bolts: ["bolt"] },
  models: [{ draw: ERendererDraw.ADDED, indices: [], positions: [], texture: "fx\\fx_lightning", uvs: [] }],
  settings: {
    altitude: [0.35, 0.35],
    deltaLongitude: 0.5,
    fogColor: 0.1,
    minDistance: 0.94,
    secondProbability: 0.5,
    skyColor: 0.1,
    sunColor: 0.9,
    tilt: 0.3,
  },
};

function createPlayer(random: () => number = Math.random): {
  player: WeatherPlayer;
  held: Set<string>;
  textures: RendererTextures;
} {
  const textures: RendererTextures = new RendererTextures(
    () => {},
    () => {}
  );
  const held: Set<string> = new Set();

  jest.spyOn(textures, "put").mockImplementation((key: string) => void held.add(key));
  jest.spyOn(textures, "release").mockImplementation((key: string) => void held.delete(key));

  return { held, player: new WeatherPlayer(textures, random), textures };
}

describe("WeatherPlayer", () => {
  it("lights nothing while no weather plays", () => {
    const { player } = createPlayer();

    expect(player.advance(0, ORIGIN)).toBeNull();
    expect(player.isPlaying).toBe(false);
    expect(player.report).toBeNull();
  });

  it("runs the clock by the time factor, and stops it while paused", () => {
    const { player } = createPlayer();

    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl(PLAYING);
    player.advance(1_000, ORIGIN);
    player.advance(1_500, ORIGIN);

    expect(player.report?.time).toBeCloseTo(6 * 3600 + 30, 6);

    player.setControl({ ...PLAYING, isPaused: true, time: null });

    expect(player.advance(2_000, ORIGIN)).not.toBeNull();
    expect(player.advance(3_000, ORIGIN)).toBeNull();
    expect(player.report?.time).toBeCloseTo(6 * 3600 + 30, 6);
  });

  it("holds the skies of the keyframes mixed and of the next, and lets the rest go", () => {
    const { player, held } = createPlayer();

    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true, time: 7 * 3600 });
    player.advance(0, ORIGIN);

    // Between six and noon, with nine at night next.
    expect([...held].sort()).toEqual(
      [
        ...new Set(
          [KEYFRAMES[1], KEYFRAMES[2], KEYFRAMES[3]].flatMap((keyframe: IRendererWeatherKeyframe) => [
            keyframe.skyTexture,
            keyframe.skyTextureEnv,
            keyframe.cloudsTexture,
          ])
        ),
      ]
        .map((reference: string) => `@weather/${reference}`)
        .sort()
    );

    player.take(null, ERendererWeatherTransition.CUT);

    expect(held.size).toBe(0);
  });

  it("lights the scene in renderer space, fogged and swaying as the view asks", () => {
    const { player } = createPlayer();

    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true, time: 12 * 3600 });

    const lit: Nullable<IRendererLighting> = player.advance(0, ORIGIN);
    const noon: IRendererWeatherKeyframe = KEYFRAMES[2];
    const [x, y, z] = noon.sunDirection ?? [0, 0, 0];

    expect(lit?.sunDirection[0]).toBeCloseTo(x, 5);
    expect(lit?.sunDirection[1]).toBeCloseTo(y, 5);
    expect(lit?.sunDirection[2]).toBeCloseTo(-z, 5);
    expect(lit?.sky.textures).toEqual([`@weather/${KEYFRAMES[1].skyTexture}`, `@weather/${noon.skyTexture}`]);
    expect(lit?.sky.blend).toBe(1);
    expect(lit?.sky.clouds).toEqual({
      color: noon.cloudsColor,
      rotation: expect.closeTo((noon.cloudsRotation * 180) / Math.PI, 4),
      textures: [`@weather/${KEYFRAMES[1].cloudsTexture}`, `@weather/${noon.cloudsTexture}`],
    });
    expect(lit?.fog).toEqual({
      color: noon.fogColor,
      density: noon.fogDensity,
      distance: noon.fogDistance,
      farPlane: noon.farPlane,
    });

    player.setControl({
      ...PLAYING,
      isClouded: false,
      isFogged: false,
      isPaused: true,
      isRainy: false,
      isWindy: false,
      time: null,
    });

    const plain: Nullable<IRendererLighting> = player.advance(0, ORIGIN);

    expect(plain?.fog).toBeNull();
    expect(plain?.trees).toBeNull();
    expect(plain?.grass).toBeNull();
    expect(plain?.sky.clouds?.textures).toEqual([null, null]);
    expect(plain?.rain).toBeNull();
  });

  it("dims the dynamic sun below the horizon", () => {
    const { player } = createPlayer();

    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isDynamicSun: true, isPaused: true, time: 0 });

    expect(player.advance(0, ORIGIN)?.sunColor).toEqual([0, 0, 0]);
  });

  it("plays an effect over the cycle until it gives the cycle back, and ends it on a seek", () => {
    const { player } = createPlayer();
    const effect: ReadonlyArray<IRendererWeatherKeyframe> = [0, 60].map((time: number) => ({
      ...KEYFRAMES[0],
      fogDistance: 5,
      time,
    }));

    player.take({ ...WEATHER, effects: { fx_test: effect } }, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, factor: 60, time: 43_000 });
    player.advance(0, ORIGIN);
    player.playEffect("fx_test");
    player.advance(0, ORIGIN);

    expect(player.report?.effect).toEqual({ name: "fx_test", remaining: expect.any(Number) });
    expect(player.report?.effect).not.toBeNull();

    // Five real seconds of lead-in at sixty to one, then a minute of game time to the effect's own keyframe.
    for (let second: number = 1; second <= 5; second += 1) {
      player.advance(second * 1000, ORIGIN);
    }

    expect(player.advance(6_000, ORIGIN)?.fog?.distance).toBeCloseTo(5, 6);

    player.setControl({ ...PLAYING, time: 43_000 });
    player.advance(6_500, ORIGIN);

    expect(player.report?.effect).toBeNull();
    expect(player.report?.effect).toBeNull();
  });

  it("weighs the level's modifiers from where the camera stands", () => {
    const { player } = createPlayer();

    player.take(
      {
        ...WEATHER,
        modifiers: [
          {
            ambient: [0, 0, 0],
            farPlane: 1000,
            fogColor: [0, 0, 0],
            fogDensity: 0,
            flags: 1,
            hemiColor: [0, 0, 0],
            position: [0, 0, 0],
            power: 1,
            radius: 10,
            skyColor: [0, 0, 0],
          },
        ],
      },
      ERendererWeatherTransition.CUT
    );
    player.setControl({ ...PLAYING, isPaused: true, time: 43_200 });

    const inside: Nullable<IRendererLighting> = player.advance(0, ORIGIN);

    expect(inside?.fog?.farPlane).toBeCloseTo((KEYFRAMES[2].farPlane + 1000) / 2, 3);
    expect(player.report?.modifiers).toBe(1);

    expect(player.advance(100, [20, 0, 0])?.fog?.farPlane).toBeCloseTo(KEYFRAMES[2].farPlane, 3);
    expect(player.report?.modifiers).toBe(0);
    // A camera barely moved weighs nothing again.
    expect(player.advance(200, [20.1, 0, 0])).toBeNull();
  });

  it("rains as hard as the keyframes say where the weather draws rain, holding its textures while it plays", () => {
    const { player, held } = createPlayer();
    const rainy: IRendererWeather = {
      ...WEATHER,
      rain: { drop: null, streak: "fx\\fx_rain" },
      textures: { ...WEATHER.textures, ["fx\\fx_rain"]: SOURCE },
    };

    player.take(rainy, ERendererWeatherTransition.CUT);
    // Nine at night, where the fixture rains.
    player.setControl({ ...PLAYING, isPaused: true, time: 21 * 3600 });

    const lit: Nullable<IRendererLighting> = player.advance(0, ORIGIN);

    expect(lit?.rain).toEqual({
      color: KEYFRAMES[3].rainColor,
      density: KEYFRAMES[3].rainDensity,
      windDirection: KEYFRAMES[3].windDirection,
      windVelocity: KEYFRAMES[3].windVelocity,
    });
    expect(held.has("@weather/fx\\fx_rain")).toBe(true);

    // Noon is dry.
    player.setControl({ ...PLAYING, isPaused: true, time: 12 * 3600 });

    expect(player.advance(100, ORIGIN)?.rain).toBeNull();
  });

  it("fades from what was shown into another weather it is handed, over the transition's time", () => {
    const { player } = createPlayer();
    const foggy: IRendererWeather = {
      ...WEATHER,
      keyframes: KEYFRAMES.map((keyframe: IRendererWeatherKeyframe) => ({ ...keyframe, fogDistance: 10 })),
    };

    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true, time: 12 * 3600 });

    const before: number = player.advance(0, ORIGIN)?.fog?.distance ?? 0;

    player.take(foggy, ERendererWeatherTransition.FADE);

    expect(player.advance(100, ORIGIN)?.fog?.distance).toBeCloseTo(before, 6);
    expect(player.advance(850, ORIGIN)?.fog?.distance).toBeCloseTo((before + 10) / 2, 6);
    expect(player.advance(1_600, ORIGIN)?.fog?.distance).toBeCloseTo(10, 6);
    // Done, and still: nothing to light again.
    expect(player.advance(1_700, ORIGIN)).toBeNull();
  });

  it("fades into the pair an effect starts on and out of the one it ends on, and cuts on a seek", () => {
    const { player } = createPlayer();
    const effect: ReadonlyArray<IRendererWeatherKeyframe> = [0, 60].map((time: number) => ({
      ...KEYFRAMES[0],
      skyTexture: "sky\\storm",
      time,
    }));

    player.take({ ...WEATHER, effects: { fx_test: effect } }, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, factor: 60, isPaused: true, time: 30_000 });

    const cycle: Nullable<IRendererLighting> = player.advance(0, ORIGIN);

    player.playEffect("fx_test");

    // The lead-in keeps the sky shown, which a paused clock then holds.
    expect(player.advance(100, ORIGIN)?.sky).toEqual(cycle?.sky);

    const led: Nullable<IRendererLighting> = player.advance(1_700, ORIGIN);

    expect(player.advance(1_800, ORIGIN)).toBeNull();

    // Ending it sets the cycle's pair after its end, which is faded into from the sky shown.
    player.playEffect(null);

    expect(player.advance(1_900, ORIGIN)?.sky).toEqual(led?.sky);

    const ended: Nullable<IRendererLighting> = player.advance(3_500, ORIGIN);

    expect(ended?.sky.textures).not.toEqual(led?.sky.textures);

    player.setControl({ ...PLAYING, factor: 60, isPaused: true, time: 30_000 });

    expect(player.advance(2_000, ORIGIN)?.sky).toEqual(cycle?.sky);
  });

  // The store calls a key never put settled, so the skies are put before the fade asks.
  it("puts the skies a fade goes to before it asks whether they are up", () => {
    const { player, textures, held } = createPlayer();
    const uploaded: Set<string> = new Set();
    const night: IRendererWeather = {
      ...WEATHER,
      keyframes: KEYFRAMES.map((keyframe: IRendererWeatherKeyframe) => ({
        ...keyframe,
        fogDistance: 10,
        skyTexture: `${keyframe.skyTexture}_night`,
      })),
      textures: Object.fromEntries(
        KEYFRAMES.map((keyframe: IRendererWeatherKeyframe) => [`${keyframe.skyTexture}_night`, SOURCE])
      ),
    };

    jest.spyOn(textures, "isUploaded").mockImplementation((key: string) => !held.has(key) || uploaded.has(key));
    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true, time: 12 * 3600 });

    const before: number = player.advance(0, ORIGIN)?.fog?.distance ?? 0;

    held.forEach((key: string) => uploaded.add(key));
    player.take(night, ERendererWeatherTransition.FADE);

    expect(player.advance(100, ORIGIN)?.fog?.distance).toBeCloseTo(before, 6);
    expect(player.advance(900, ORIGIN)?.fog?.distance).toBeCloseTo(before, 6);
  });

  it("starts a fade once the skies it fades into are up, or two seconds after it was asked for", () => {
    const { player, textures } = createPlayer();
    const night: IRendererWeather = {
      ...WEATHER,
      keyframes: KEYFRAMES.map((keyframe: IRendererWeatherKeyframe) => ({ ...keyframe, fogDistance: 10 })),
    };
    const uploaded: jest.SpiedFunction<(key: string) => boolean> = jest
      .spyOn(textures, "isUploaded")
      .mockReturnValue(false);

    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true, time: 12 * 3600 });

    const before: number = player.advance(0, ORIGIN)?.fog?.distance ?? 0;

    player.take(night, ERendererWeatherTransition.FADE);

    expect(player.advance(100, ORIGIN)?.fog?.distance).toBeCloseTo(before, 6);
    expect(player.advance(1_000, ORIGIN)?.fog?.distance).toBeCloseTo(before, 6);

    uploaded.mockReturnValue(true);
    player.advance(1_100, ORIGIN);

    expect(player.advance(1_850, ORIGIN)?.fog?.distance).toBeCloseTo((before + 10) / 2, 6);

    // Where they never go up, it starts without them.
    uploaded.mockReturnValue(false);
    player.take(WEATHER, ERendererWeatherTransition.FADE);
    player.advance(3_000, ORIGIN);
    player.advance(5_000, ORIGIN);

    expect(player.advance(5_750, ORIGIN)?.fog?.distance).toBeGreaterThan(10);
  });

  it("hands the cycle back the keyframes an effect ends on, and reports what it blends", () => {
    const { player } = createPlayer();
    const effect: ReadonlyArray<IRendererWeatherKeyframe> = [0, 60].map((time: number) => ({ ...KEYFRAMES[0], time }));

    player.take({ ...WEATHER, effects: { fx_test: effect } }, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, factor: 60, time: 43_000 });
    player.advance(0, ORIGIN);

    expect(player.report?.between).toEqual([21_600, 43_200]);

    player.playEffect("fx_test");

    for (let second: number = 1; second <= 12; second += 1) {
      player.advance(second * 1000, ORIGIN);
    }

    // `WFX_end_desc` is nine at night, the first at or after the effect's last, then midnight after it. The next
    // `SelectEnvs` finds that pair across midnight with the clock between its two, so it moves on to midnight and nine
    // at night, as the engine's does.
    expect(player.report?.effect).toBeNull();
    expect(player.report?.between).toEqual([0, 75_600]);
  });

  it("lights a one-keyframe weather as that keyframe whatever the time, and reports it as the current keyframe", () => {
    const { player } = createPlayer();
    const noon: IRendererWeatherKeyframe = KEYFRAMES[2];

    player.take({ ...WEATHER, keyframes: [noon] }, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true, time: 3_000 });

    expect(player.advance(0, ORIGIN)?.fog?.distance).toBeCloseTo(noon.fogDistance, 6);
    expect(player.report?.current).toMatchObject({
      fogDistance: noon.fogDistance,
      skyTexture: noon.skyTexture,
      time: 3_000,
    });
  });

  // Noon strikes with the bolts every ten seconds for half a second; the clock stands still, real time runs.
  it("strikes on a paused clock, lighting every frame of the strike and the frame after it ends", () => {
    const { player, held } = createPlayer(() => 0.5);
    const noon: IRendererWeatherKeyframe = KEYFRAMES[2];

    player.take(
      {
        ...WEATHER,
        keyframes: [noon],
        textures: { ...WEATHER.textures, [THUNDER.models[0].texture]: SOURCE },
        thunder: THUNDER,
      },
      ERendererWeatherTransition.CUT
    );
    player.setControl({ ...PLAYING, isPaused: true, time: 43_200 });

    const calm: Nullable<IRendererLighting> = player.advance(0, ORIGIN);

    expect(calm?.thunderbolt).toBeNull();
    expect(held).toContain("@weather/fx\\fx_lightning");
    expect(player.advance(5_000, ORIGIN)).toBeNull();
    expect(player.advance(9_990, ORIGIN)).toBeNull();

    const struck: Nullable<IRendererLighting> = player.advance(10_010, ORIGIN);

    expect(struck?.thunderbolt?.bolt).toBe("bolt");
    // Lit white: the sun by 0.9 of it, the sky by a tenth, clamped, and the sun turned to come down from the bolt.
    expect(struck?.sunColor[0]).toBeCloseTo((calm?.sunColor[0] ?? 0) + 0.9, 5);
    expect(struck?.sky.color[0]).toBeCloseTo(Math.min((calm?.sky.color[0] ?? 0) + 0.1, 1), 5);
    expect(struck?.sunDirection[1]).toBeLessThan(0);

    const frames: Array<Nullable<IRendererLighting>> = [10_110, 10_210, 10_310, 10_410, 10_510, 10_610].map(
      (now: number) => player.advance(now, ORIGIN)
    );

    expect(frames.every((frame) => frame?.thunderbolt)).toBe(true);
    expect(player.advance(10_710, ORIGIN)).toEqual(calm);
    expect(player.advance(10_810, ORIGIN)).toBeNull();
  });

  // Anomaly's `sky2` takes the sky through the tonemap's curve; vanilla's does not.
  it("draws the sky through the curve on the extended engine alone", () => {
    const { player } = createPlayer();

    player.take(WEATHER, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true });

    expect(player.advance(0, ORIGIN)?.sky.isCurved).toBe(false);

    player.take({ ...WEATHER, engine: ERendererWeatherEngine.EXTENDED }, ERendererWeatherTransition.CUT);

    expect(player.advance(100, ORIGIN)?.sky.isCurved).toBe(true);
  });

  it("strikes nothing while thunder is switched off", () => {
    const { player } = createPlayer(() => 0.5);

    player.take({ ...WEATHER, keyframes: [KEYFRAMES[2]], thunder: THUNDER }, ERendererWeatherTransition.CUT);
    player.setControl({ ...PLAYING, isPaused: true, isThundering: false, time: 43_200 });
    player.advance(0, ORIGIN);

    expect([5_000, 10_010, 10_200, 20_000].map((now: number) => player.advance(now, ORIGIN))).toEqual([
      null,
      null,
      null,
      null,
    ]);
  });
});
