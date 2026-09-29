import { describe, expect, it } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { TRendererVector } from "#/contract/renderer-vector";
import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { IRendererThunder } from "#/contract/weather/renderer-thunder";
import { IWeatherMix } from "#/weather/weather-mix";
import { WeatherThunder } from "#/weather/weather-thunder";
import { IWeatherThunderFlash } from "#/weather/weather-thunder-flash";

/** Vanilla's `[environment]`, in radians. */
const THUNDER: IRendererThunder = {
  animators: [
    {
      colors: [
        [255, 128, 0],
        [0, 0, 0],
      ],
      fps: 2,
      frameCount: 2,
      frames: [0, 1],
    },
  ],
  bolts: {
    bolt: {
      center: { draw: ERendererDraw.ADDED, opacity: 0.2, radius: [2, 1], texture: "fx\\fx_thunderbolts_gradient" },
      color: 0,
      model: 0,
      top: { draw: ERendererDraw.ADDED, opacity: 0.6, radius: [0.5, 0.25], texture: "fx\\fx_thunderbolts_gradient" },
    },
  },
  collections: { bolts: ["bolt"], empty: [] },
  models: [{ draw: ERendererDraw.ADDED, indices: [], positions: [], texture: "fx\\fx_lightning", uvs: [] }],
  settings: {
    altitude: [(20 * Math.PI) / 180, (20 * Math.PI) / 180],
    deltaLongitude: (30 * Math.PI) / 180,
    fogColor: 0.1,
    minDistance: 0.94,
    secondProbability: 0.5,
    skyColor: 0.1,
    sunColor: 0.9,
    tilt: (17 * Math.PI) / 180,
  },
};

/** The sun in the south-east, low, as it travels; strikes every ten seconds for half a second. */
const MIX = {
  farPlane: 1000,
  sunDirection: [0.6, -0.4, -0.7],
  thunderboltCollection: "bolts",
  thunderboltDuration: 0.5,
  thunderboltPeriod: 10,
} as unknown as IWeatherMix;

const VIEW: TRendererVector = [10, 30, -20];

function run(
  thunder: WeatherThunder,
  times: ReadonlyArray<number>,
  mix: IWeatherMix = MIX
): Array<Nullable<IWeatherThunderFlash>> {
  return times.map((now: number) => thunder.advance({ isEnabled: true, mix, now, thunder: THUNDER, view: VIEW }));
}

describe("WeatherThunder", () => {
  // At a half every roll: due a period after the weather starts to strike, the next a period after that.
  it("strikes a period after the weather names a collection, for as long as the strike lasts", () => {
    const thunder: WeatherThunder = new WeatherThunder(() => 0.5);
    const flashes = run(thunder, [0, 5, 9.99, 10.01, 10.3, 10.6, 10.62, 15]);

    expect(flashes.slice(0, 3)).toEqual([null, null, null]);
    // Idle once its half second has run, the frame it ends still lit.
    expect(flashes.slice(3, 7).every((flash) => flash !== null)).toBe(true);
    expect(flashes.slice(7)).toEqual([null]);
    expect(run(thunder, [20.5])[0]).not.toBeNull();
  });

  it("strikes across the sky from the sun, at the far plane, down to the ground", () => {
    const flash = run(new WeatherThunder(() => 0.5), [0, 11])[1] as IWeatherThunderFlash;
    const { position, axes } = flash.strike;
    const [ox, oy, oz] = [position[0] - VIEW[0], position[1] - VIEW[1], position[2] - VIEW[2]];
    const bottom: number = position[1] - axes[1][1];

    // Opposite the sun's heading, twenty degrees up, between 0.94 and 0.95 of the far plane.
    expect(Math.atan2(-ox, oz)).toBeCloseTo(Math.atan2(-0.6, -0.7) + Math.PI, 5);
    expect(Math.asin(oy / Math.hypot(ox, oy, oz))).toBeCloseTo((20 * Math.PI) / 180, 5);
    expect(Math.hypot(ox, oy, oz)).toBeCloseTo(945, 3);
    expect(bottom).toBeCloseTo(0, 3);
    // The sun turned to come from the bolt.
    expect(flash.direction[1]).toBeLessThan(0);
    expect(flash.strike.bolt).toBe("bolt");
  });

  it("lights by the bolt's colour animation over its life, its top glow's opacity on both glows", () => {
    const flashes = run(new WeatherThunder(() => 0.5), [0, 9.99, 10.01, 10.26]);
    const [, , first, second] = flashes as Array<IWeatherThunderFlash>;

    // Its first frame, then past halfway through its life, at 0.27 of its half second.
    expect(first.color).toEqual([1, 128 / 255, 0]);
    expect(second.color).toEqual([0, 0, 0]);
    expect(second.strike.top.opacity).toBeCloseTo(0.6 * 1.5 * (0.27 / 0.5), 5);
    expect(second.strike.center.opacity).toBeCloseTo(0.6 * 1.5 * (0.27 / 0.5), 5);
    expect(second.strike.center.extent).toEqual([2 * second.strike.axes[1][1], second.strike.axes[1][1]].map(Math.abs));
  });

  it("strikes nothing without a collection, or with one of no bolts, or while switched off", () => {
    const empty: IWeatherMix = { ...MIX, thunderboltCollection: "empty" };
    const none: IWeatherMix = { ...MIX, thunderboltCollection: null };
    const off: WeatherThunder = new WeatherThunder(() => 0.5);

    expect(run(new WeatherThunder(() => 0.5), [0, 11, 30], empty)).toEqual([null, null, null]);
    expect(run(new WeatherThunder(() => 0.5), [0, 11, 30], none)).toEqual([null, null, null]);
    expect(
      [0, 11, 30].map((now: number) => off.advance({ isEnabled: false, mix: MIX, now, thunder: THUNDER, view: VIEW }))
    ).toEqual([null, null, null]);
  });

  it("follows a strike at once by the second probability", () => {
    // Every roll a quarter: under the half, so the next is due as soon as this one ends.
    const flashes = run(new WeatherThunder(() => 0.25), [0, 7.6, 7.8, 8.0, 8.2]);

    expect(flashes[1]).not.toBeNull();
    expect(flashes[4]).not.toBeNull();
  });
});
