import { describe, expect, it } from "@jest/globals";

import { ETextureBumpPlane, toTextureBumpPlane } from "@/core/textures/lib/texture-bump-plane";
import { ITextureBumpTexels } from "@/core/textures/lib/texture-surface";

/** One texel a half: the bump `(gloss, z, y, x)` and the companion `(error x, y, z, height)`, as bytes. */
function mockTexels(bump: Array<number>, companion: Array<number>): ITextureBumpTexels {
  return {
    bump: { data: new Uint8Array(bump), height: 1, width: 1 },
    companion: { data: new Uint8Array(companion), height: 1, width: 1 },
  };
}

describe("toTextureBumpPlane", () => {
  const texels: ITextureBumpTexels = mockTexels([128, 255, 128, 64], [255, 255, 255, 200]);

  it("shows both files as stored, opaque", () => {
    expect([...toTextureBumpPlane(texels, ETextureBumpPlane.BUMP)]).toEqual([128, 255, 128, 255]);
    expect([...toTextureBumpPlane(texels, ETextureBumpPlane.COMPANION)]).toEqual([255, 255, 255, 255]);
  });

  it("reconstructs the normal from the bump's alpha, blue and green, corrected by the companion", () => {
    // `Nu.wzy + (NuE.xyz - 1)`: a full error leaves the stored values, mapped from the signed range into the unit one.
    // 64 and 128 land on 159.5 and 191.5, which a clamped byte rounds to even.
    expect([...toTextureBumpPlane(texels, ETextureBumpPlane.NORMAL)].slice(0, 3)).toEqual([160, 192, 255]);
  });

  it("squares the bump's red into gloss and reads the companion's alpha as height", () => {
    expect([...toTextureBumpPlane(texels, ETextureBumpPlane.GLOSS)].slice(0, 3)).toEqual([64, 64, 64]);
    expect([...toTextureBumpPlane(texels, ETextureBumpPlane.HEIGHT)].slice(0, 3)).toEqual([200, 200, 200]);
  });
});
