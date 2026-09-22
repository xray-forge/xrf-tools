import { describe, expect, it } from "@jest/globals";

import { EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { EDdsChannels } from "#/texture/dds/dds-channels";
import { getDdsDxgiLayout } from "#/texture/dds/dds-dxgi";
import { EDdsLayout } from "#/texture/dds/dds-layout";

describe("getDdsDxgiLayout", () => {
  // The single commonest layout the example loader refused: plain DXT5 wearing a DX10 header, which is 3,679 files of
  // the project's own resource pack.
  it("reads BC3 under a dx10 header as dxt5", () => {
    expect(getDdsDxgiLayout(77)).toEqual({ blockBytes: 16, format: EDdsBlockFormat.BC3, kind: EDdsLayout.BLOCK });
  });

  it("reads BC7, which the reference trees ship thirty of", () => {
    expect(getDdsDxgiLayout(98)).toMatchObject({ format: EDdsBlockFormat.BC7 });
  });

  it("reads BC5, which is how a bump pair is stored", () => {
    // Worse than cosmetic before: the png fallback is deliberately skipped for bump pairs, so an ATI2 bump was
    // silently unshaded rather than degraded.
    expect(getDdsDxgiLayout(83)).toMatchObject({ format: EDdsBlockFormat.BC5 });
  });

  it("reads the two uncompressed orders as texels", () => {
    expect(getDdsDxgiLayout(28)).toEqual({ channels: EDdsChannels.RGBA, kind: EDdsLayout.TEXELS });
    expect(getDdsDxgiLayout(87)).toEqual({ channels: EDdsChannels.BGRA, kind: EDdsLayout.TEXELS });
  });

  it("names nothing for a code it does not model", () => {
    expect(getDdsDxgiLayout(1)).toBeNull();
  });
});
