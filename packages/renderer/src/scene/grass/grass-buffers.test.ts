import { describe, expect, it } from "@jest/globals";

import { toGrassCachePerCell, toGrassItemCapacity } from "#/scene/grass/grass-buffers";

describe("toGrassCachePerCell", () => {
  it("holds every candidate a slot lays out while the cache fits one storage buffer", () => {
    expect(toGrassCachePerCell(49 * 49, 25, 1 << 27)).toBe(25);
  });

  it("holds fewer where every candidate would not fit, and one at the least", () => {
    // 32 bytes a tuft: 100 slots in 32,000 bytes hold ten each.
    expect(toGrassCachePerCell(100, 25, 32_000)).toBe(10);
    expect(toGrassCachePerCell(100_000, 25, 32_000)).toBe(1);
  });
});

describe("toGrassItemCapacity", () => {
  it("rounds a need up to the next power of two, so a setting dragged up rebuilds the lists rarely", () => {
    expect(toGrassItemCapacity(1, 1 << 27)).toBe(1);
    expect(toGrassItemCapacity(1000, 1 << 27)).toBe(1024);
    expect(toGrassItemCapacity(1024, 1 << 27)).toBe(1024);
    expect(toGrassItemCapacity(0, 1 << 27)).toBe(1);
  });

  it("rounds no further than one storage buffer holds, even for a need past it", () => {
    // 32 bytes an item: a limit of 32,000 bytes holds 1,000.
    expect(toGrassItemCapacity(600, 32_000)).toBe(1000);
    expect(toGrassItemCapacity(2000, 32_000)).toBe(1000);
  });
});
