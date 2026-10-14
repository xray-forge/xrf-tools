import { describe, expect, it } from "@jest/globals";

import { parseHexColor, toRawColor } from "#/color";

describe("toRawColor", () => {
  it("reads each byte of a hex colour as a raw value", () => {
    expect(toRawColor(0xff8000)).toEqual([1, 128 / 255, 0]);
  });
});

describe("parseHexColor", () => {
  it("reads a css hex colour as one number", () => {
    expect(parseHexColor("#ff8000")).toBe(0xff8000);
    expect(parseHexColor("#1A1A1A")).toBe(0x1a1a1a);
  });
});
