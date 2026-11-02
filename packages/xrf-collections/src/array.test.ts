import { describe, expect, it } from "@jest/globals";

import { EMPTY_ARRAY, withoutAt, withValueAt } from "#/array";

describe("array", () => {
  it("shares one empty list that refuses entries", () => {
    expect(EMPTY_ARRAY).toHaveLength(0);
    expect(() => (EMPTY_ARRAY as unknown as Array<number>).push(1)).toThrow(TypeError);
  });

  it("copies a list without one entry, unchanged past its end", () => {
    const items: Array<string> = ["a", "b", "c"];

    expect(withoutAt(items, 1)).toEqual(["a", "c"]);
    expect(withoutAt(items, 3)).toEqual(["a", "b", "c"]);
    expect(items).toEqual(["a", "b", "c"]);
  });

  it("copies a list with one entry replaced, unchanged past its end", () => {
    const items: Array<string> = ["a", "b"];

    expect(withValueAt(items, 0, "z")).toEqual(["z", "b"]);
    expect(withValueAt(items, 5, "z")).toEqual(["a", "b"]);
    expect(items).toEqual(["a", "b"]);
  });
});
