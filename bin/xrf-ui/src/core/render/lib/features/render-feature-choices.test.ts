import { describe, expect, it } from "@jest/globals";

import {
  formatContactShadowLights,
  formatLowLuminance,
  formatShadowBias,
  formatShadowReach,
  formatWaveSpeed,
  fromGrassDensityScale,
  RENDER_FRAME_RATE_OPTIONS,
  RENDER_GRASS_LIMITS,
  RENDER_RESOLUTION_OPTIONS,
  toGrassDensityScale,
} from "@/core/render/lib/features/render-feature-choices";
import { DEFAULT_RENDER_GRASS_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { FRAME_RATE_LIMITS } from "@/core/render/lib/settings/render-frame-rate-limit";
import { RENDER_RESOLUTIONS } from "@/core/render/lib/settings/render-resolution";

describe("grass density scale", () => {
  // The engine's density is a spacing: 0.99 its sparsest and 0.6 its own; the settings stop at 0.2, three times it.
  it("offers the engine's density as how many times the game's the grass stands, higher denser", () => {
    expect(toGrassDensityScale(DEFAULT_RENDER_GRASS_SETTINGS.density)).toBe(1);
    expect(toGrassDensityScale(0.3)).toBeCloseTo(2, 10);
    expect(RENDER_GRASS_LIMITS.density.max).toBeCloseTo(3, 10);
    expect(fromGrassDensityScale(RENDER_GRASS_LIMITS.density.max)).toBeCloseTo(0.2, 10);
    expect(fromGrassDensityScale(RENDER_GRASS_LIMITS.density.min)).toBeCloseTo(0.99, 10);
    expect(fromGrassDensityScale(toGrassDensityScale(0.45))).toBeCloseTo(0.45, 10);
  });
});

describe("render choice formatters", () => {
  it("reads each value in the unit and the precision it is set in", () => {
    expect(formatShadowBias(0.25)).toBe("0.25");
    expect(formatShadowReach(150)).toBe("150 m");
    expect(formatWaveSpeed(12)).toBe("12");
    expect(formatLowLuminance(0.0001)).toBe("0.0001");
    expect(formatContactShadowLights(0)).toBe("Sun only");
    expect(formatContactShadowLights(1)).toBe("1 light");
    expect(formatContactShadowLights(4)).toBe("4 lights");
  });
});

describe("render choice options", () => {
  it("offers every frame rate limit and resolution this build takes, in its order", () => {
    expect(RENDER_FRAME_RATE_OPTIONS.map((it) => it.value)).toEqual(FRAME_RATE_LIMITS);
    expect(RENDER_FRAME_RATE_OPTIONS[0]?.label).toBe("No cap");
    expect(RENDER_RESOLUTION_OPTIONS.map((it) => it.value)).toEqual(RENDER_RESOLUTIONS);
    expect(RENDER_RESOLUTION_OPTIONS.map((it) => it.label)).toEqual(["Window", "720p", "1080p", "1440p", "4K"]);
  });
});
