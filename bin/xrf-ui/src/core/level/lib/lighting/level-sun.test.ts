import { describe, expect, it } from "@jest/globals";

import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toSunAngles } from "@/core/level/lib/lighting/level-sun";
import { toRenderSunPosition } from "@/core/render/lib/lighting/render-lighting";

function lighting(overrides: Partial<ILevelLighting> = {}): ILevelLighting {
  return { ...DEFAULT_LEVEL_LIGHTING, ...overrides };
}

describe("toSunAngles", () => {
  // The chunk stores where the light travels, so a sun overhead is a light going straight down.
  it("puts a sun overhead for a light travelling straight down", () => {
    expect(toSunAngles({ x: 0, y: -1, z: 0 })?.elevation).toBeCloseTo(90);
  });

  it("answers nothing for a level that aimed its sun nowhere", () => {
    expect(toSunAngles({ x: 0, y: 0, z: 0 })).toBeNull();
    expect(toSunAngles(null)).toBeNull();
  });

  // A component the wire left short is not a direction, and inventing one would aim the light somewhere arbitrary.
  it("reads a short vector as the zeroes it carries", () => {
    expect(toSunAngles({ x: null, y: null, z: null })).toBeNull();
  });

  // Round trip: an angle put into a position and read back out of the direction it implies comes back the same.
  it("is the inverse of the position it places", () => {
    const placed = toRenderSunPosition(lighting({ sunAzimuth: 35, sunElevation: 55 }), 1);
    // The renderer negates `z` between the level's axes and its own, and this travels from the sun rather than to it.
    const angles = toSunAngles({ x: -placed[0], y: -placed[1], z: placed[2] });

    expect(angles?.elevation).toBeCloseTo(55);
    expect(angles?.azimuth).toBeCloseTo(35);
  });
});
