import { describe, expect, it } from "@jest/globals";

import { toGrassItemCapacity } from "#/scene/grass/grass-buffers";

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
