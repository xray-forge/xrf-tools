import { describe, expect, it } from "@jest/globals";

import { DEFAULT_LEVEL_LIGHTING, ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toRenderSunPosition } from "@/core/render/lib/lighting/render-lighting";

function lighting(overrides: Partial<ILevelLighting> = {}): ILevelLighting {
  return { ...DEFAULT_LEVEL_LIGHTING, ...overrides };
}

describe("toRenderSunPosition", () => {
  it("puts a sun overhead directly above, whatever it is turned to", () => {
    const [x, y, z] = toRenderSunPosition(lighting({ sunAzimuth: 123, sunElevation: 90 }), 100);

    expect(x).toBeCloseTo(0);
    expect(y).toBeCloseTo(100);
    expect(z).toBeCloseTo(0);
  });

  // Zero looks along the renderer's own `+z`, which is what the azimuth is stated against.
  it("puts a sun on the horizon along the axis its azimuth names", () => {
    const [x, y, z] = toRenderSunPosition(lighting({ sunAzimuth: 0, sunElevation: 0 }), 100);

    expect(x).toBeCloseTo(0);
    expect(y).toBeCloseTo(0);
    expect(z).toBeCloseTo(100);

    const turned = toRenderSunPosition(lighting({ sunAzimuth: 90, sunElevation: 0 }), 100);

    expect(turned[0]).toBeCloseTo(100);
    expect(turned[2]).toBeCloseTo(0);
  });

  it("puts it as far out as it is asked to", () => {
    const [, y] = toRenderSunPosition(lighting({ sunElevation: 90 }), 4200);

    expect(y).toBeCloseTo(4200);
  });
});
