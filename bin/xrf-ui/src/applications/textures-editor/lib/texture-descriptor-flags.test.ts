import { describe, expect, it } from "@jest/globals";

import { hasTextureFlag, withTextureFlag } from "./texture-descriptor-flags";

describe("texture flag words", () => {
  it("should read and write one bit", () => {
    expect(hasTextureFlag(0b101, 0b100)).toBe(true);
    expect(hasTextureFlag(0b001, 0b100)).toBe(false);
    expect(withTextureFlag(0b001, 0b100, true)).toBe(0b101);
    expect(withTextureFlag(0b101, 0b100, false)).toBe(0b001);
  });

  it("should leave bits the sdk never named exactly as they were", () => {
    // A word a third-party tool wrote bits into keeps them. Dropping a bit nobody here understands is the silent
    // rewrite this editor exists to avoid.
    const unnamed: number = 1 << 30;

    expect(withTextureFlag(unnamed, 1, true)).toBe(unnamed | 1);
    expect(withTextureFlag(unnamed | 1, 1, false)).toBe(unnamed);
  });
});
