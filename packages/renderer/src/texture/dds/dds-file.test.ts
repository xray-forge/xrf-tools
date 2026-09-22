import { describe, expect, it } from "@jest/globals";

import { EDdsBlockFormat } from "#/texture/dds/dds-block-format";
import { EDdsChannels } from "#/texture/dds/dds-channels";
import { IDdsFile, IDdsRead, readDdsFile } from "#/texture/dds/dds-file";
import { mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "#/texture/dds/dds-fixtures";
import { EDdsLayout } from "#/texture/dds/dds-layout";
import { EDdsRefusal, IDdsRefusal } from "#/texture/dds/dds-refusal";

/** The file a read produced, failing the case rather than the assertion when it was refused. */
function readFile(bytes: ArrayBuffer): IDdsFile {
  const read: IDdsRead = readDdsFile(bytes);

  if (!read.file) {
    throw new Error(`expected a readable file, got ${read.refusal?.reason}: ${read.refusal?.detail}`);
  }

  return read.file;
}

/** Why a read refused, failing the case rather than the assertion when it did not. */
function refusalOf(bytes: ArrayBuffer): IDdsRefusal {
  const read: IDdsRead = readDdsFile(bytes);

  if (read.file || !read.refusal) {
    throw new Error("expected the file to be refused");
  }

  return read.refusal;
}

describe("readDdsFile", () => {
  it("reads a dxt1 file with its mip chain", () => {
    const file: IDdsFile = readFile(mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 3, width: 4 }));

    expect(file.width).toBe(4);
    expect(file.mipmaps).toHaveLength(3);
    expect(file.layout).toEqual({ blockBytes: 8, format: EDdsBlockFormat.BC1, kind: EDdsLayout.BLOCK });
  });

  it("hands out block bytes as a view rather than a copy", () => {
    // The blocks go to the gpu exactly as stored, and copying them would double what a texture costs on the way in.
    const bytes: ArrayBuffer = mockDdsFile({ fourCC: "DXT5" });

    expect(readFile(bytes).mipmaps[0].data.buffer).toBe(bytes);
  });

  it("reads a layout from the DX10 header when the file carries one", () => {
    expect(readFile(mockDx10DdsFile(77)).layout).toMatchObject({ format: EDdsBlockFormat.BC3 });
  });

  it("expands a texel layout end to end", () => {
    const file: IDdsFile = readFile(mockUncompressedDdsFile({ height: 1, texels: [[10, 20, 30, 40]], width: 1 }));

    expect(file.layout).toEqual({ channels: EDdsChannels.BGRA, kind: EDdsLayout.TEXELS });
    // Stored blue, green, red, alpha; read back red first.
    expect(Array.from(file.mipmaps[0].data)).toEqual([30, 20, 10, 40]);
  });

  it("takes a picture of exactly one block", () => {
    expect(readFile(mockDdsFile({ fourCC: "DXT1", height: 4, width: 4 })).width).toBe(4);
  });
});

describe("readDdsFile refusals", () => {
  it("says which dxgi code, tag or masks it does not model", () => {
    // The category is what a caller matches on; the detail is what a report shows a person.
    expect(refusalOf(mockDx10DdsFile(1))).toEqual({
      detail: "DXGI_FORMAT 1 is not modelled",
      reason: EDdsRefusal.UNSUPPORTED_DXGI,
    });
    expect(refusalOf(mockDdsFile({ fourCC: "YUY2" }))).toEqual({
      detail: "the four character tag 'YUY2' is not modelled",
      reason: EDdsRefusal.UNSUPPORTED_FOURCC,
    });

    const masks: IDdsRefusal = refusalOf(mockUncompressedDdsFile({ bitCount: 16, blueMask: 0x001f, redMask: 0xf800 }));

    expect(masks.reason).toBe(EDdsRefusal.UNSUPPORTED_MASKS);
    expect(masks.detail).toContain("16 bit");
  });

  it("refuses a texture array or a volume, which a surface has no way to draw", () => {
    expect(refusalOf(mockDx10DdsFile(77, { arraySize: 6 })).reason).toBe(EDdsRefusal.UNSUPPORTED_DIMENSION);
    expect(refusalOf(mockDx10DdsFile(77, { resourceDimension: 4 })).reason).toBe(EDdsRefusal.UNSUPPORTED_DIMENSION);
  });

  it("refuses a cubemap where it recognises one, and says whether its faces are all there", () => {
    // Refused by the reader rather than by the upload above it: six faces are not a surface texture whichever way
    // they are drawn, so there is no readable outcome to hand on.
    const whole: ArrayBuffer = mockDdsFile();
    const partial: ArrayBuffer = mockDdsFile();

    new Uint32Array(whole)[28] = 0x200 | 0x400 | 0x800 | 0x1000 | 0x2000 | 0x4000 | 0x8000;
    new Uint32Array(partial)[28] = 0x200 | 0x400;

    expect(refusalOf(whole)).toEqual({ detail: "the file is a cubemap, six faces", reason: EDdsRefusal.CUBEMAP });
    expect(refusalOf(partial).detail).toContain("missing faces");
  });

  it("refuses a file that stops before the texels its header declares", () => {
    // A view over a buffer that is one block short throws; refusing it is what keeps a bad file from taking the app.
    const complete: ArrayBuffer = mockDdsFile({ fourCC: "DXT5", height: 8, mipmapCount: 1, width: 8 });

    expect(refusalOf(complete.slice(0, complete.byteLength - 1)).reason).toBe(EDdsRefusal.TRUNCATED);
  });

  // The defect this exists for: WebGL answers `INVALID_OPERATION` and WebGPU invalidates the texture for a block
  // compressed base level narrower than its block, and a texture that failed to upload samples as black. Anomaly
  // ships seven leaf sprays as 2x2 fully transparent DXT1 files to switch that geometry off, and they drew as solid
  // black cards. Direct3D takes them, which is why the game does not show this.
  it("refuses a block compressed picture smaller than one block, so the backend expands it instead", () => {
    const refusal: IDdsRefusal = refusalOf(mockDdsFile({ fourCC: "DXT1", height: 2, width: 2 }));

    expect(refusal.reason).toBe(EDdsRefusal.SUB_BLOCK);
    expect(refusal.detail).toContain("2x2");
  });
});
