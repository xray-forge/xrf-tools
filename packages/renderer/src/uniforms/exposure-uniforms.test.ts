import { describe, expect, it } from "@jest/globals";
import { uniform } from "three/tsl";

import { DEFAULT_RENDERER_EXPOSURE_SETTINGS } from "#/contract/renderer-exposure-settings";
import { ExposureUniforms } from "#/uniforms/exposure-uniforms";

describe("ExposureUniforms", () => {
  // `phase_luminance`: `MiddleGray` is `(1, 0, 1)` lerped towards `(middle gray, 1, low luminance)` by the amount.
  it("sets the engine's MiddleGray from the settings", () => {
    const exposure: ExposureUniforms = new ExposureUniforms(uniform(1));

    exposure.apply({ ...DEFAULT_RENDERER_EXPOSURE_SETTINGS, amount: 1, lowLuminance: 0.5, middleGray: 1.5 });

    expect(exposure.target.value).toBeCloseTo(1.5);
    expect(exposure.weight.value).toBeCloseTo(1);
    expect(exposure.floor.value).toBeCloseTo(0.5);

    exposure.apply(DEFAULT_RENDERER_EXPOSURE_SETTINGS);

    expect(exposure.target.value).toBeCloseTo(1);
    expect(exposure.weight.value).toBeCloseTo(0.7);
    expect(exposure.floor.value).toBeCloseTo(0.3 + 0.7 * 0.0001);
  });

  it("moves its rate as f_luminance_adapt does, and goes back to one when nothing adapts", () => {
    const exposure: ExposureUniforms = new ExposureUniforms(uniform(1));

    exposure.advance(0.1, 2);

    expect(exposure.blend.value).toBeCloseTo(0.9 * 0.5 + 0.1 * 0.1 * 2);

    (exposure.adapted.array as Float32Array)[0] = 3;
    exposure.reset();

    expect((exposure.adapted.array as Float32Array)[0]).toBe(1);
  });
});
