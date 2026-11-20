import { describe, expect, it } from "@jest/globals";

import { EMPTY_SET } from "#/set";

describe("set", () => {
  it("shares one empty set that refuses entries", () => {
    expect(EMPTY_SET.size).toBe(0);
    expect(() => (EMPTY_SET as unknown as Set<number>).add(1)).toThrow(TypeError);
    expect(EMPTY_SET.size).toBe(0);
  });

  it("reads and spreads as any empty set", () => {
    expect(EMPTY_SET.has(1 as never)).toBe(false);
    expect([...EMPTY_SET]).toEqual([]);
  });
});
