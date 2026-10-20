import { describe, expect, it } from "@jest/globals";

import { isRecord } from "#/is-record";

describe("isRecord", () => {
  it("takes an object read by key, and nothing else", () => {
    expect(isRecord({})).toBe(true);
    expect(isRecord({ a: 1 })).toBe(true);
    expect(isRecord(null)).toBe(false);
    expect(isRecord([1, 2])).toBe(false);
    expect(isRecord("a")).toBe(false);
    expect(isRecord(undefined)).toBe(false);
  });
});
