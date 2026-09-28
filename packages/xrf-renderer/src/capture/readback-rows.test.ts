import { describe, expect, it } from "@jest/globals";

import { READBACK_ROW_ALIGNMENT, unpadReadbackRows } from "#/capture/readback-rows";

describe("unpadReadbackRows", () => {
  it("drops each row's padding up to the copy alignment", () => {
    // Two texels a row is eight bytes, padded to a whole alignment.
    const padded: Uint8Array = new Uint8Array(READBACK_ROW_ALIGNMENT * 2);

    padded.set([1, 2, 3, 4, 5, 6, 7, 8], 0);
    padded.set([9, 10, 11, 12, 13, 14, 15, 16], READBACK_ROW_ALIGNMENT);

    expect(Array.from(unpadReadbackRows(padded, 2, 2))).toEqual([
      1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ]);
  });

  it("copies rows already a whole alignment wide as they are", () => {
    const width: number = READBACK_ROW_ALIGNMENT / 4;
    const rows: Uint8Array = Uint8Array.from({ length: READBACK_ROW_ALIGNMENT * 2 }, (_, index: number) => index % 251);

    expect(Array.from(unpadReadbackRows(rows, width, 2))).toEqual(Array.from(rows));
  });
});
