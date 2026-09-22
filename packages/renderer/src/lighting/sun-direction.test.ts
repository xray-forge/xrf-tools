import { describe, expect, it } from "@jest/globals";

import { toRendererSunDirection } from "#/lighting/sun-direction";

describe("toRendererSunDirection", () => {
  // Noon of `default_clear` says altitude -69 and longitude -30. Through `setHP(altitude, longitude)` that is a sun
  // thirty degrees up, not sixty-nine: the longitude is the pitch.
  it("reads the longitude as how high the sun stands", () => {
    const [x, y, z] = toRendererSunDirection(-69, -30);

    expect(y).toBeCloseTo(-0.5, 6);
    expect(x).toBeCloseTo(0.808504, 6);
    expect(z).toBeCloseTo(-0.310356, 6);
  });

  it("is a unit vector", () => {
    expect(Math.hypot(...toRendererSunDirection(40, -55))).toBeCloseTo(1, 10);
  });
});
