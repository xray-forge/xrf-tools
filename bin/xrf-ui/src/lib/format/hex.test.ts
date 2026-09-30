import { describe, expect, it } from "@jest/globals";

import { formatHex } from "@/lib/format/hex";

describe("formatHex", () => {
  it("writes the word in upper case hex, unpadded by default", () => {
    expect(formatHex(0xff00)).toBe("0xFF00");
  });

  it("pads to the digits asked", () => {
    expect(formatHex(0xff00, 8)).toBe("0x0000FF00");
  });

  it("reads a negative word as unsigned 32 bits", () => {
    expect(formatHex(-1)).toBe("0xFFFFFFFF");
  });
});
