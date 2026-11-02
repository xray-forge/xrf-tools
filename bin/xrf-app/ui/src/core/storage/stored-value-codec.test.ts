import { describe, expect, it } from "@jest/globals";

import { booleanCodec } from "@/core/storage/stored-value-codec";

describe("booleanCodec", () => {
  it("reads the flag stored, and the fallback for anything else", () => {
    for (const [stored, fallback, expected] of [
      [null, true, true],
      ["false", true, false],
      ["true", false, true],
      ["yes", true, true],
      ["yes", false, false],
    ] as const) {
      expect(booleanCodec(fallback).read(stored)).toBe(expected);
    }
  });

  it("writes the flag as text", () => {
    expect(booleanCodec(true).write(false)).toBe("false");
  });
});
