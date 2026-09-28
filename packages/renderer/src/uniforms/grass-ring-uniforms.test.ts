import { describe, expect, it } from "@jest/globals";

import { GrassRingUniforms } from "#/uniforms/grass-ring-uniforms";

describe("GrassRingUniforms", () => {
  it("reaches as far as the planting where its build holds the bands, and counts the square of cells it covers", () => {
    const ring: GrassRingUniforms = new GrassRingUniforms();

    ring.fit(3, 10);

    expect(ring.reach.value).toBe(3);
    expect(ring.cells).toBe(49);
  });

  it("reaches no further than its build's bands, however far the planting reaches", () => {
    const ring: GrassRingUniforms = new GrassRingUniforms();

    ring.fit(49, 26);

    expect(ring.reach.value).toBe(25);
    expect(ring.cells).toBe(51 * 51);
  });
});
