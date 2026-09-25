import { describe, expect, it } from "@jest/globals";

import { createGrassDither } from "#/scene/grass/grass-planting.tsl";

describe("createGrassDither", () => {
  // `bwdithermap(2, dither)`: `magic4x4` spread over sixteen by sixteen, the coarse cell's value scaled by 254 / 16
  // and the fine one's by a sixteenth of that.
  it("lays out the engine's thresholds, column by row", () => {
    const dither: Uint32Array = createGrassDither();

    expect(dither).toHaveLength(256);
    expect(dither[0]).toBe(0);
    // `magic[0][1]`: `magic4x4[0][1]`, 14, over a coarse cell of 15.875.
    expect(dither[1]).toBe(222);
    // `magic[1][0]`: `magic4x4[1][0]`, 11, coarse steps and nothing finer.
    expect(dither[16]).toBe(175);
    // `magic[4][0]`: nothing coarse, and a sixteenth of `magic4x4[1][0]`'s eleven steps.
    expect(dither[4 * 16]).toBe(11);
    // `magic[10][10]`: both cells at `magic4x4[2][2]`, 15.
    expect(dither[10 * 16 + 10]).toBe(253);
    expect(Math.max(...dither)).toBe(253);
  });
});
