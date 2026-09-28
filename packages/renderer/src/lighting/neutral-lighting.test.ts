import { describe, expect, it } from "@jest/globals";

import { TRendererColor } from "#/contract/renderer-color";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { NEUTRAL_RENDERER_LIGHTING } from "#/lighting/neutral-lighting";

function isGrey([red, green, blue]: TRendererColor): boolean {
  return red === green && green === blue;
}

describe("NEUTRAL_RENDERER_LIGHTING", () => {
  it("turns every colour grey and keeps the direction", () => {
    const neutral: IRendererLighting = NEUTRAL_RENDERER_LIGHTING;

    expect([neutral.sunColor, neutral.hemisphereColor, neutral.skyIrradiance, neutral.ambientColor].every(isGrey)).toBe(
      true
    );
    expect(neutral.sunDirection).toBe(DEFAULT_RENDERER_LIGHTING.sunDirection);
  });

  it("keeps each colour's luminance, so the neutral light is as bright as noon", () => {
    const [red, green, blue] = DEFAULT_RENDERER_LIGHTING.sunColor;

    expect(NEUTRAL_RENDERER_LIGHTING.sunColor[0]).toBeCloseTo(0.2126 * red + 0.7152 * green + 0.0722 * blue);
  });

  it("is as unfogged as noon", () => {
    expect(NEUTRAL_RENDERER_LIGHTING.fog).toBeNull();
  });
});
