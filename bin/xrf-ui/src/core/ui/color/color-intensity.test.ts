import { describe, expect, it } from "@jest/globals";

import { fromPickedColor, rescaleColor, toColorIntensity, toPickedColor } from "@/core/ui/color/color-intensity";

describe("color intensity", () => {
  it("keeps a colour within one as it is, and scales an overbright one into the picker's range", () => {
    expect(toColorIntensity([0.5, 0.25, 1])).toBe(1);
    expect(toColorIntensity([2, 1, 0.5])).toBe(2);
    expect(toPickedColor([2, 1, 0.5], 2)).toEqual({ b: 64, g: 128, r: 255 });
  });

  // An engine sun is often brighter than one: a pick keeps its brightness, and only the hue and shade change.
  it("gives a pick back at the colour's own brightness", () => {
    const [r, g, b] = fromPickedColor({ b: 0, g: 255, r: 255 }, 2);

    expect([r, g, b]).toEqual([2, 2, 0]);
  });

  it("rescales a colour to the brightness asked for", () => {
    expect(rescaleColor([2, 1, 0.5], 2, 3)).toEqual([3, 1.5, 0.75]);
  });
});
