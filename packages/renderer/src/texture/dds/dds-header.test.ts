import { describe, expect, it } from "@jest/globals";

import { mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "#/texture/dds/dds-fixtures";
import { IDdsHeader, readDdsHeader } from "#/texture/dds/dds-header";
import { EDdsRefusal } from "#/texture/dds/dds-refusal";

/** The header a read produced, failing the case rather than the assertion when it was refused. */
function readHeader(bytes: ArrayBuffer): IDdsHeader {
  const { header, refusal } = readDdsHeader(bytes);

  if (!header) {
    throw new Error(`expected a readable header, got ${refusal?.reason}: ${refusal?.detail}`);
  }

  return header;
}

describe("readDdsHeader", () => {
  it("reads the size, the chain and the tag", () => {
    const header: IDdsHeader = readHeader(mockDdsFile({ fourCC: "DXT5", height: 8, mipmapCount: 4, width: 16 }));

    expect(header).toMatchObject({ extended: null, fourCc: "DXT5", height: 8, mipmapCount: 4, width: 16 });
    expect(header.dataOffset).toBe(128);
  });

  it("leaves the tag empty for a layout stored as channel masks", () => {
    const header: IDdsHeader = readHeader(mockUncompressedDdsFile());

    expect(header.fourCc).toBe("");
    expect(header.masks.bitCount).toBe(32);
  });

  it("reads the extended header and starts the texels after it", () => {
    const header: IDdsHeader = readHeader(mockDx10DdsFile(77, { arraySize: 6 }));

    expect(header.extended).toEqual({ arraySize: 6, dimension: 3, dxgiFormat: 77 });
    expect(header.dataOffset).toBe(148);
  });

  it("says whether a cubemap has all six faces", () => {
    const whole: ArrayBuffer = mockDdsFile();
    const partial: ArrayBuffer = mockDdsFile();

    new Uint32Array(whole)[28] = 0x200 | 0x400 | 0x800 | 0x1000 | 0x2000 | 0x4000 | 0x8000;
    new Uint32Array(partial)[28] = 0x200 | 0x400;

    expect(readHeader(whole).cubemap).toEqual({ isWhole: true });
    expect(readHeader(partial).cubemap).toEqual({ isWhole: false });
    expect(readHeader(mockDdsFile()).cubemap).toBeNull();
  });

  it("tells a file that is not a dds from one that stops short", () => {
    // The two used to look identical from outside, and they are opposite fixes.
    expect(readDdsHeader(new Uint8Array([1, 2, 3, 4]).buffer).refusal?.reason).toBe(EDdsRefusal.TRUNCATED);

    const notADds: ArrayBuffer = mockDdsFile();

    new Int32Array(notADds)[0] = 0;

    expect(readDdsHeader(notADds).refusal?.reason).toBe(EDdsRefusal.NOT_A_DDS);
  });

  it("refuses a file that declares a DX10 header and stops before it", () => {
    expect(readDdsHeader(mockDx10DdsFile(77).slice(0, 130)).refusal?.reason).toBe(EDdsRefusal.TRUNCATED);
  });
});
