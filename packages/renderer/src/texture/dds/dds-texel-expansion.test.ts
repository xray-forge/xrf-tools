import { describe, expect, it } from "@jest/globals";

import { EDdsChannels } from "#/texture/dds/dds-channels";
import { expandDdsTexels } from "#/texture/dds/dds-texel-expansion";

/** Bytes laid out as a file stores them, starting at a nonzero offset so the offset is exercised too. */
function stored(bytes: ReadonlyArray<number>): ArrayBuffer {
  return new Uint8Array([0xee, 0xee, 0xee, 0xee, ...bytes]).buffer;
}

describe("expandDdsTexels", () => {
  it("reorders bgra into rgba", () => {
    expect(Array.from(expandDdsTexels(stored([30, 20, 10, 40]), 4, 1, 1, EDdsChannels.BGRA))).toEqual([10, 20, 30, 40]);
  });

  it("copies rgba as stored", () => {
    expect(Array.from(expandDdsTexels(stored([10, 20, 30, 40]), 4, 1, 1, EDdsChannels.RGBA))).toEqual([10, 20, 30, 40]);
  });

  it("makes bgr opaque, since three bytes say nothing about alpha", () => {
    expect(Array.from(expandDdsTexels(stored([30, 20, 10, 60, 50, 40]), 4, 2, 1, EDdsChannels.BGR))).toEqual([
      10, 20, 30, 255, 40, 50, 60, 255,
    ]);
  });
});
