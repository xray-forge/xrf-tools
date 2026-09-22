import { describe, expect, it } from "@jest/globals";

import { EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { getDdsFourCcLayout, toDdsFourCc } from "#/texture/dds/dds-fourcc";
import { EDdsLayout } from "#/texture/dds/dds-layout";

describe("toDdsFourCc", () => {
  it("spells a tag low byte first", () => {
    expect(toDdsFourCc(0x31545844)).toBe("DXT1");
  });
});

describe("getDdsFourCcLayout", () => {
  it("names the s3tc families, with the byte size of their blocks", () => {
    expect(getDdsFourCcLayout("DXT1")).toEqual({ blockBytes: 8, format: EDdsBlockFormat.BC1, kind: EDdsLayout.BLOCK });
    expect(getDdsFourCcLayout("DXT3")).toMatchObject({ blockBytes: 16, format: EDdsBlockFormat.BC2 });
    expect(getDdsFourCcLayout("DXT5")).toMatchObject({ blockBytes: 16, format: EDdsBlockFormat.BC3 });
  });

  it("takes both spellings the sdk writes a bump plane under", () => {
    expect(getDdsFourCcLayout("ATI1")).toMatchObject({ format: EDdsBlockFormat.BC4 });
    expect(getDdsFourCcLayout("BC4U")).toMatchObject({ format: EDdsBlockFormat.BC4 });
    expect(getDdsFourCcLayout("ATI2")).toMatchObject({ format: EDdsBlockFormat.BC5 });
    expect(getDdsFourCcLayout("BC5S")).toMatchObject({ format: EDdsBlockFormat.BC5_SIGNED });
  });

  it("names nothing for a tag it does not model", () => {
    expect(getDdsFourCcLayout("YUY2")).toBeNull();
  });
});
