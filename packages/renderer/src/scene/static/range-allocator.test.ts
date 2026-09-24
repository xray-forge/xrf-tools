import { describe, expect, it } from "@jest/globals";

import { RangeAllocator } from "#/scene/static/range-allocator";

describe("RangeAllocator", () => {
  it("hands runs out first fit, and nothing once full", () => {
    const allocator: RangeAllocator = new RangeAllocator();

    allocator.grow(10);

    expect(allocator.allocate(4)).toBe(0);
    expect(allocator.allocate(4)).toBe(4);
    expect(allocator.allocate(4)).toBeNull();
    expect(allocator.allocate(2)).toBe(8);
  });

  it("joins a released run with its free neighbours, so a longer run fits again", () => {
    const allocator: RangeAllocator = new RangeAllocator();

    allocator.grow(12);
    allocator.allocate(4);
    allocator.allocate(4);
    allocator.allocate(4);
    allocator.release(0, 4);
    allocator.release(8, 4);
    allocator.release(4, 4);

    expect(allocator.allocate(12)).toBe(0);
  });

  it("frees what growing adds, joined to a free run at the end", () => {
    const allocator: RangeAllocator = new RangeAllocator();

    allocator.grow(4);
    allocator.allocate(2);
    allocator.grow(8);

    expect(allocator.capacity).toBe(8);
    expect(allocator.allocate(6)).toBe(2);
  });

  it("reaches as far as the end of its last run handed out", () => {
    const allocator: RangeAllocator = new RangeAllocator();

    allocator.grow(10);

    expect(allocator.extent).toBe(0);

    allocator.allocate(4);
    allocator.allocate(4);
    allocator.release(0, 4);

    expect(allocator.extent).toBe(8);

    // First fit: the freed run first, then the end.
    allocator.allocate(4);
    allocator.allocate(2);

    expect(allocator.extent).toBe(10);
  });
});
