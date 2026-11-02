import { describe, expect, it } from "@jest/globals";

import { IRenderLighting, RENDER_LIGHTING_LIMITS } from "@/core/render/lib/lighting/render-lighting";
import {
  DEFAULT_TEXTURE_LIGHTING,
  dragTextureLighting,
  TEXTURE_LIGHT_ELEVATION_LIMIT,
} from "@/core/textures/lib/texture-lighting";

describe("dragTextureLighting", () => {
  it("swings the light by the share of the viewport dragged across", () => {
    const lighting: IRenderLighting = dragTextureLighting(DEFAULT_TEXTURE_LIGHTING, 50, 20, 100, 100);

    expect(lighting.sunAzimuth).toBeCloseTo(DEFAULT_TEXTURE_LIGHTING.sunAzimuth + 90);
    expect(lighting.sunElevation).toBeCloseTo(DEFAULT_TEXTURE_LIGHTING.sunElevation - 36);
  });

  // The sliders show what the pointer set, so the pointer never leaves where they can follow.
  it("wraps the azimuth round into the sliders' range", () => {
    const lighting: IRenderLighting = dragTextureLighting(
      { ...DEFAULT_TEXTURE_LIGHTING, sunAzimuth: 170 },
      20,
      0,
      100,
      100
    );

    expect(lighting.sunAzimuth).toBeCloseTo(-154);
    expect(
      dragTextureLighting({ ...DEFAULT_TEXTURE_LIGHTING, sunAzimuth: -170 }, -20, 0, 100, 100).sunAzimuth
    ).toBeCloseTo(154);
  });

  it("holds the elevation between the sliders' floor and short of the zenith", () => {
    expect(dragTextureLighting(DEFAULT_TEXTURE_LIGHTING, 0, 1000, 100, 100).sunElevation).toBe(
      RENDER_LIGHTING_LIMITS.sunElevation.min
    );
    expect(dragTextureLighting(DEFAULT_TEXTURE_LIGHTING, 0, -1000, 100, 100).sunElevation).toBe(
      TEXTURE_LIGHT_ELEVATION_LIMIT
    );
  });
});
