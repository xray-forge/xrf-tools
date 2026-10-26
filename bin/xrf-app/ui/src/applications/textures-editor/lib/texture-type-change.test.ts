import { describe, expect, it } from "@jest/globals";
import { TextureTypeChange } from "@xrf/ipc/types/xrf-app-textures";

import { TextureTypePatch, withTextureType } from "./texture-type-change";

const TERRAIN: TextureTypeChange = {
  clearedFlags: 0b11,
  format: 0,
  mipFilter: null,
  setFlags: 1 << 24,
  textureType: 4,
};
const NORMAL_MAP: TextureTypeChange = { clearedFlags: 0, format: 6, mipFilter: 14, setFlags: 1, textureType: 3 };

const FORM: TextureTypePatch = { flags: 0b111, format: 4, mipFilter: 1, textureType: 0 };

describe("withTextureType", () => {
  it("should clear and set the type's flags and take its format", () => {
    expect(withTextureType(FORM, 4, [TERRAIN, NORMAL_MAP])).toEqual({
      flags: 0b100 | (1 << 24),
      format: 0,
      mipFilter: 1,
      textureType: 4,
    });
  });

  it("should take the type's mip filter", () => {
    expect(withTextureType(FORM, 3, [TERRAIN, NORMAL_MAP])).toEqual({
      flags: 0b111,
      format: 6,
      mipFilter: 14,
      textureType: 3,
    });
  });

  it("should store a type with no named change alone", () => {
    expect(withTextureType(FORM, 9, [TERRAIN])).toEqual({ ...FORM, textureType: 9 });
  });

  it("should keep a set top bit unsigned", () => {
    const top: number = 2 ** 31;

    expect(withTextureType({ ...FORM, flags: top }, 4, [TERRAIN]).flags).toBe(top + (1 << 24));
  });
});
