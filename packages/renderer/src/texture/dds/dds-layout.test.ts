import { describe, expect, it } from "@jest/globals";

import { EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { EDdsChannels } from "#/texture/dds/dds-channels";
import { getDdsStoredLength, toDdsBlockLayout, toDdsTexelLayout } from "#/texture/dds/dds-layout";

describe("getDdsStoredLength", () => {
  it("costs a block layout whole blocks, and never less than one", () => {
    const layout = toDdsBlockLayout(EDdsBlockFormat.BC1, 8);

    expect(getDdsStoredLength(layout, 8, 8)).toBe(32);
    // A chain runs below one block, and the smallest levels still cost a whole one.
    expect(getDdsStoredLength(layout, 2, 1)).toBe(8);
  });

  it("costs a texel layout what the file stores, not what it expands to", () => {
    expect(getDdsStoredLength(toDdsTexelLayout(EDdsChannels.BGR), 2, 2)).toBe(12);
    expect(getDdsStoredLength(toDdsTexelLayout(EDdsChannels.BGRA), 2, 2)).toBe(16);
  });
});
