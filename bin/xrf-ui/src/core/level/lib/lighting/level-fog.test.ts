import { describe, expect, it } from "@jest/globals";

import { DEFAULT_LEVEL_FOG, toLevelRendererFog } from "@/core/level/lib/lighting/level-fog";

describe("toLevelRendererFog", () => {
  it("is default_clear's noon fog as it stands", () => {
    expect(toLevelRendererFog(DEFAULT_LEVEL_FOG)).toEqual({
      color: [0.304609, 0.328138, 0.367354],
      density: 0.9,
      distance: 350,
    });
  });

  it("scales the noon colour and takes the distance and density as set", () => {
    const fog = toLevelRendererFog({ fogDensity: 0.5, fogDistance: 800, fogIntensity: 2 });

    expect(fog.color[0]).toBeCloseTo(0.609218);
    expect([fog.distance, fog.density]).toEqual([800, 0.5]);
  });
});
