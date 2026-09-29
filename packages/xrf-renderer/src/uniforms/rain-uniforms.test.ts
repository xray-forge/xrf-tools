import { describe, expect, it } from "@jest/globals";
import { Vector3 } from "three/webgpu";

import { RAIN_STREAKS, RainUniforms } from "#/uniforms/rain-uniforms";

describe("RainUniforms", () => {
  it("draws half the streaks at the lightest rain and all of them at the heaviest, covering more as it thickens", () => {
    const rain: RainUniforms = new RainUniforms();

    rain.take({ color: [0.5, 1.5, 0.2], density: 0, windDirection: 0, windVelocity: 0 });

    expect(rain.count.value).toBe(RAIN_STREAKS / 2);
    expect(rain.color.value.toArray()).toEqual([0.5, 1, 0.2, 0.5]);

    rain.take({ color: [0.5, 0.5, 0.5], density: 1, windDirection: 0, windVelocity: 0 });

    expect(rain.count.value).toBe(RAIN_STREAKS);
    expect(rain.color.value.w).toBe(1);
    expect(rain.isFalling).toBe(true);

    rain.take(null);

    expect(rain.isFalling).toBe(false);
    expect(rain.count.value).toBe(0);
  });

  // `drop_max_angle` at `drop_max_wind_vel` of wind with the gust at a twentieth: a calm falls straight down.
  it("leans the rain with the wind, ten degrees at most, towards the heading it blows to", () => {
    const rain: RainUniforms = new RainUniforms();

    rain.take({ color: [1, 1, 1], density: 0.5, windDirection: 0, windVelocity: 0 });

    expect(rain.axis.value.distanceTo(new Vector3(0, -1, 0))).toBeCloseTo(0, 6);

    rain.take({ color: [1, 1, 1], density: 0.5, windDirection: Math.PI / 2, windVelocity: 10_000 });

    expect(rain.axis.value.angleTo(new Vector3(0, -1, 0))).toBeCloseTo((10 * Math.PI) / 180, 6);
    // Heading a quarter turn: `setHP` leans it along engine `-x`, which renderer space keeps.
    expect(rain.axis.value.x).toBeLessThan(0);
    expect(rain.axis.value.z).toBeCloseTo(0, 6);
  });

  it("points the streaks at the cover once it is drawn where it now stands", () => {
    const rain: RainUniforms = new RainUniforms();

    rain.cover.fit(new Vector3(9, 3, -7));
    rain.commitCover();

    expect(rain.window.value.toArray()).toEqual([8, -8, 28, 64]);
  });
});
