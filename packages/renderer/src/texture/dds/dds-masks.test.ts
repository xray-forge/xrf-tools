import { describe, expect, it } from "@jest/globals";

import { EDdsChannels } from "#/texture/dds/dds-channels";
import { EDdsLayout } from "#/texture/dds/dds-layout";
import { describeDdsMasks, getDdsMaskLayout, IDdsChannelMasks } from "#/texture/dds/dds-masks";

const A8R8G8B8: IDdsChannelMasks = {
  alpha: 0xff000000,
  bitCount: 32,
  blue: 0x000000ff,
  green: 0x0000ff00,
  red: 0x00ff0000,
};

describe("getDdsMaskLayout", () => {
  it("reads A8R8G8B8 as bgra", () => {
    expect(getDdsMaskLayout(A8R8G8B8)).toEqual({ channels: EDdsChannels.BGRA, kind: EDdsLayout.TEXELS });
  });

  it("reads A8B8G8R8 as rgba, told apart from A8R8G8B8 by the whole mask", () => {
    // Anomaly ships 24 references to `A8B8G8R8`. Its red sits in the low byte, which an overlap test would miss.
    expect(getDdsMaskLayout({ ...A8R8G8B8, blue: 0x00ff0000, red: 0x000000ff })).toEqual({
      channels: EDdsChannels.RGBA,
      kind: EDdsLayout.TEXELS,
    });
  });

  it("reads R8G8B8 as bgr", () => {
    expect(getDdsMaskLayout({ ...A8R8G8B8, alpha: 0, bitCount: 24 })).toEqual({
      channels: EDdsChannels.BGR,
      kind: EDdsLayout.TEXELS,
    });
  });

  it("names nothing for channels that are not whole bytes", () => {
    // `R5G6B5`: unpacking it is decoding rather than reordering, which is the backend's job.
    expect(getDdsMaskLayout({ alpha: 0, bitCount: 16, blue: 0x001f, green: 0x07e0, red: 0xf800 })).toBeNull();
  });
});

describe("describeDdsMasks", () => {
  it("names the bit count and every mask in hex", () => {
    expect(describeDdsMasks({ alpha: 0, bitCount: 16, blue: 0x001f, green: 0x07e0, red: 0xf800 })).toBe(
      "an uncompressed 16 bit layout, r=0x0000f800 g=0x000007e0 b=0x0000001f a=0x00000000"
    );
  });
});
