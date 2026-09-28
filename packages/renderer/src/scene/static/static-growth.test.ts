import { describe, expect, it, jest } from "@jest/globals";
import { Nullable } from "@xrf/types";

import { RangeAllocator } from "#/scene/static/range-allocator";
import { allocateGrowing, toFittedCapacity, toGrownCapacity } from "#/scene/static/static-growth";

function createFragmented(): RangeAllocator {
  const allocator: RangeAllocator = new RangeAllocator();

  allocator.grow(100);

  for (let run = 0; run < 10; run += 1) {
    allocator.allocate(10);
  }

  for (let start = 0; start < 100; start += 20) {
    allocator.release(start, 10);
  }

  return allocator;
}

describe("toGrownCapacity", () => {
  it("grows to what is used and wanted, with headroom", () => {
    expect(toGrownCapacity(100, 60, 100, 16, 1000)).toBe(200);
  });

  it("grows by at least one, to at least the initial size, and never past the limit", () => {
    expect(toGrownCapacity(0, 0, 100, 16, 1000)).toBe(101);
    expect(toGrownCapacity(0, 1, 0, 16, 1000)).toBe(16);
    expect(toGrownCapacity(900, 400, 900, 16, 1000)).toBe(1000);
  });
});

describe("toFittedCapacity", () => {
  it("keeps what a buffer holds where a free run fits, asking nothing of the queue", () => {
    const allocator: RangeAllocator = new RangeAllocator();
    const toWanted = jest.fn(() => 1000);

    allocator.grow(10);

    expect(toFittedCapacity(allocator, 10, 1, 1000, toWanted)).toBe(10);
    expect(toWanted).not.toHaveBeenCalled();
  });

  it("grows for what is used and what the queue brings, with headroom", () => {
    const allocator: RangeAllocator = new RangeAllocator();

    allocator.grow(10);
    allocator.allocate(10);

    expect(toFittedCapacity(allocator, 10, 1, 1000, () => 20)).toBe(50);
  });

  // Freed room in runs too short would have grown the space by one element a pass, a whole copy each time.
  it("grows a fragmented space past the end of its last run by the whole run, in one step", () => {
    const allocator: RangeAllocator = createFragmented();

    expect(allocator.used).toBe(50);
    expect(allocator.extent).toBe(100);
    expect(toFittedCapacity(allocator, 20, 1, 1000)).toBe(150);
  });

  it("refuses a run the limit leaves no room for, rather than growing short of it", () => {
    expect(toFittedCapacity(createFragmented(), 20, 1, 110)).toBeNull();
  });
});

describe("allocateGrowing", () => {
  it("grows a fragmented space once, to where the run fits after its last", () => {
    const allocator: RangeAllocator = createFragmented();
    const grow = jest.fn((capacity: number) => allocator.grow(capacity));
    const start: Nullable<number> = allocateGrowing(allocator, 20, 1, 1000, grow);

    expect(grow).toHaveBeenCalledTimes(1);
    expect(grow).toHaveBeenCalledWith(150);
    expect(start).toBe(100);
  });

  it("grows nothing where a run fits, or where the limit stops it", () => {
    const grow = jest.fn();
    const allocator: RangeAllocator = createFragmented();

    expect(allocateGrowing(allocator, 10, 1, 1000, grow)).toBe(0);
    expect(allocateGrowing(allocator, 20, 1, 100, grow)).toBeNull();
    expect(grow).not.toHaveBeenCalled();
  });
});
