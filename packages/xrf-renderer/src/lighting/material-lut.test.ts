import { describe, expect, it } from "@jest/globals";

import { createMaterialLut, MATERIAL_LUT_LDOTH, MATERIAL_LUT_LDOTN } from "#/lighting/material-lut";

function mockTexel(data: Uint8Array, slice: number, x: number, y: number): [number, number] {
  const at: number = ((slice * MATERIAL_LUT_LDOTH + y) * MATERIAL_LUT_LDOTN + x) * 2;

  return [data[at], data[at + 1]];
}

describe("createMaterialLut", () => {
  const lut: Uint8Array = createMaterialLut();

  it("holds four slices of 128 by 256, two bytes a texel", () => {
    expect(lut).toHaveLength(128 * 256 * 4 * 2);
  });

  it("forces the far corner of every slice to full diffuse and specular", () => {
    for (let slice = 0; slice < 4; slice++) {
      expect(mockTexel(lut, slice, 127, 255)).toEqual([255, 255]);
    }
  });

  // Worked by hand from `r4_rendertarget_build_textures.cpp`, not by the code under test.
  it("quantises each model as the engine does", () => {
    // Oren-Nayar diffuse at N.L = 64 / 127: floor(0.50394^0.75 * 255.5).
    expect(mockTexel(lut, 0, 64, 0)[0]).toBe(152);
    // Oren-Nayar specular at N.L = 1, N.H = 250 / 255: floor(0.98039^16 * 0.5 * 255.5).
    expect(mockTexel(lut, 0, 127, 250)[1]).toBe(93);
    // Blinn at N.L = 1, N.H = 250 / 255: full diffuse, floor(0.98039^24 * 255.5) specular.
    expect(mockTexel(lut, 1, 127, 250)).toEqual([255, 158]);
    // Phong at N.L = 1 with no highlight.
    expect(mockTexel(lut, 2, 127, 0)).toEqual([255, 0]);
  });
});
