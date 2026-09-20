import { describe, expect, it } from "@jest/globals";

import { DEFAULT_LEVEL_LIGHTING, ILevelLighting, toSunAngles, toSunPosition } from "@/core/level/lib/level-lighting";

function lighting(overrides: Partial<ILevelLighting> = {}): ILevelLighting {
  return { ...DEFAULT_LEVEL_LIGHTING, ...overrides };
}

describe("toSunPosition", () => {
  it("puts a sun overhead directly above, whatever it is turned to", () => {
    const [x, y, z] = toSunPosition(lighting({ sunAzimuth: 123, sunElevation: 90 }), 100);

    expect(x).toBeCloseTo(0);
    expect(y).toBeCloseTo(100);
    expect(z).toBeCloseTo(0);
  });

  // Zero looks along the renderer's own `+z`, which is what the azimuth is stated against.
  it("puts a sun on the horizon along the axis its azimuth names", () => {
    const [x, y, z] = toSunPosition(lighting({ sunAzimuth: 0, sunElevation: 0 }), 100);

    expect(x).toBeCloseTo(0);
    expect(y).toBeCloseTo(0);
    expect(z).toBeCloseTo(100);

    const turned = toSunPosition(lighting({ sunAzimuth: 90, sunElevation: 0 }), 100);

    expect(turned[0]).toBeCloseTo(100);
    expect(turned[2]).toBeCloseTo(0);
  });

  it("puts it as far out as it is asked to", () => {
    const [, y] = toSunPosition(lighting({ sunElevation: 90 }), 4200);

    expect(y).toBeCloseTo(4200);
  });
});

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
    const placed = toSunPosition(lighting({ sunAzimuth: 35, sunElevation: 55 }), 1);
    // The renderer negates `z` between the level's axes and its own, and this travels from the sun rather than to it.
    const angles = toSunAngles({ x: -placed[0], y: -placed[1], z: placed[2] });

    expect(angles?.elevation).toBeCloseTo(55);
    expect(angles?.azimuth).toBeCloseTo(35);
  });
});
