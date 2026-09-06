import { describe, expect, it } from "@jest/globals";

import {
  describeTextureTexel,
  ITextureTexelReadout,
  toTextureTexelPosition,
} from "@/applications/textures-explorer/components/editor/panels/TextureChannelsPanel/texture-channel-readout";
import { ITextureBumpTexels } from "@/applications/textures-explorer/lib/texture-surface";
import { IVisualTextureTexels } from "@/core/visuals/lib/visual-texture";

/** A two by two plane whose four texels are told apart by their red channel. */
function mockPlane(texels: ReadonlyArray<[number, number, number, number]>): IVisualTextureTexels {
  return { data: new Uint8Array(texels.flat()), height: 2, width: 2 };
}

/**
 * A pair whose second texel carries values chosen so the decode can be written out by hand below.
 */
function mockPair(): ITextureBumpTexels {
  return {
    bump: mockPlane([
      [0, 0, 0, 0],
      // Gloss 128, and the normal reversed into (alpha, blue, green) as the packer stores it.
      [128, 40, 90, 200],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
    ]),
    companion: mockPlane([
      [0, 0, 0, 0],
      // The quantisation error of the normal's three channels, and the height in alpha where the packer puts it.
      [255, 200, 64, 20],
      [0, 0, 0, 0],
      [0, 0, 0, 0],
    ]),
  };
}

describe("describeTextureTexel", () => {
  it("states the bytes each half stores", () => {
    const readout: ITextureTexelReadout = describeTextureTexel(mockPair(), { x: 1, y: 0 });

    expect(readout.position).toBe("1, 0 of 2 x 2");
    expect(readout.bump).toBe("128, 40, 90, 200");
    expect(readout.companion).toBe("255, 200, 64, 20");
  });

  it("reconstructs the normal as the engine does, component by component", () => {
    const readout: ITextureTexelReadout = describeTextureTexel(mockPair(), { x: 1, y: 0 });

    // `Nu.wzy + (NuE.xyz - 1)`: alpha with red, blue with green, green with blue, each in the unit range.
    const x: number = 200 / 255 + (255 / 255 - 1);
    const y: number = 90 / 255 + (200 / 255 - 1);
    const z: number = 40 / 255 + (64 / 255 - 1);

    expect(readout.normal).toBe([x, y, z].map((it: number) => it.toFixed(3)).join(", "));
  });

  it("reconstructs gloss as the bump's red squared and height as the companion's alpha", () => {
    // Height is alpha, not the blue beside it: blue is the error of the normal's z and is already spent correcting
    // it above, which is the discrepancy `issues/0149` records in the engine's own loader.
    const readout: ITextureTexelReadout = describeTextureTexel(mockPair(), { x: 1, y: 0 });

    expect(readout.gloss).toBe(((128 / 255) * (128 / 255)).toFixed(3));
    expect(readout.height).toBe((20 / 255).toFixed(3));
  });

  it("reads the row a file stores first as the row at the top", () => {
    // The second row of the fixture is all zeroes, so reading it proves rows are not counted from the bottom.
    expect(describeTextureTexel(mockPair(), { x: 1, y: 1 }).bump).toBe("0, 0, 0, 0");
  });
});

describe("toTextureTexelPosition", () => {
  it("maps a fraction of a tile onto a texel of the plane", () => {
    const pair: ITextureBumpTexels = mockPair();

    expect(toTextureTexelPosition(pair, 0.1, 0.1)).toEqual({ x: 0, y: 0 });
    expect(toTextureTexelPosition(pair, 0.9, 0.1)).toEqual({ x: 1, y: 0 });
    expect(toTextureTexelPosition(pair, 0.1, 0.9)).toEqual({ x: 0, y: 1 });
  });

  it("keeps a pointer on the far edge inside the plane", () => {
    // A pointer exactly on the right or bottom edge lands one past the last index without this.
    expect(toTextureTexelPosition(mockPair(), 1, 1)).toEqual({ x: 1, y: 1 });
  });
});
