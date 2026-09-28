import { describe, expect, it } from "@jest/globals";

import {
  createGrassCacheBuffers,
  GRASS_CACHE_KEY_WORDS,
  IGrassCacheBuffers,
  toGrassCachePerCell,
} from "#/scene/grass/grass-cache-buffers";

describe("toGrassCachePerCell", () => {
  it("holds every candidate a slot lays out while the ring fits one storage buffer", () => {
    expect(toGrassCachePerCell(49 * 49, 25, 1 << 27)).toBe(25);
  });

  it("holds fewer where every candidate would not fit, and one at the least", () => {
    // 32 bytes a tuft: 100 slots in 32,000 bytes hold ten each.
    expect(toGrassCachePerCell(100, 25, 32_000)).toBe(10);
    expect(toGrassCachePerCell(100_000, 25, 32_000)).toBe(1);
  });
});

describe("createGrassCacheBuffers", () => {
  it("starts every cell stale, keyed under generation nought, which no planting runs under", () => {
    const cache: IGrassCacheBuffers = createGrassCacheBuffers(9, 25, 2);
    const keys: Uint32Array = cache.keys.array as Uint32Array;

    expect(keys).toHaveLength(9 * GRASS_CACHE_KEY_WORDS);
    expect(keys.every((word: number) => word === 0)).toBe(true);
    expect(cache.items.array).toHaveLength(9 * 25 * 2 * 4);
    expect(cache.bandCounts.array).toHaveLength(2);
  });
});
