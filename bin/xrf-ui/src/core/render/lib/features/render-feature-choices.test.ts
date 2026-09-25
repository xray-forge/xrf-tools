import { describe, expect, it } from "@jest/globals";
import { DEFAULT_RENDERER_GRASS_SETTINGS } from "@xrf/renderer";

import {
  fromGrassDensityScale,
  RENDER_GRASS_LIMITS,
  toGrassDensityScale,
} from "@/core/render/lib/features/render-feature-choices";

describe("grass density scale", () => {
  // The engine's density is a spacing: 0.1 is its densest, 0.99 its sparsest, and 0.6 its own.
  it("offers the engine's density as how many times the game's the grass stands, higher denser", () => {
    expect(toGrassDensityScale(DEFAULT_RENDERER_GRASS_SETTINGS.density)).toBe(1);
    expect(toGrassDensityScale(0.3)).toBeCloseTo(2, 10);
    expect(fromGrassDensityScale(RENDER_GRASS_LIMITS.density.max)).toBeCloseTo(0.1, 10);
    expect(fromGrassDensityScale(RENDER_GRASS_LIMITS.density.min)).toBeLessThan(0.99);
    expect(fromGrassDensityScale(toGrassDensityScale(0.45))).toBeCloseTo(0.45, 10);
  });
});
