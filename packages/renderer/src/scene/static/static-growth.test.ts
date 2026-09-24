import { describe, expect, it } from "@jest/globals";

import { toGrownCapacity } from "#/scene/static/static-growth";

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
