import { describe, expect, it } from "@jest/globals";
import {
  RED_GREEN_RGTC2_Format,
  RGB_S3TC_DXT1_Format,
  RGBA_BPTC_Format,
  RGBA_S3TC_DXT1_Format,
  RGBA_S3TC_DXT5_Format,
} from "three";

import { EDdsRefusal, IDdsFile, IDdsRead, IDdsRefusal, readDdsFile } from "@/core/render/lib/dds/dds-file";
import { EDdsChannels, EDdsLayout } from "@/core/render/lib/dds/dds-format";
import { mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "@/fixtures/mocks/dds.mocks";

/** The file a read produced, failing the case rather than the assertion when it was refused. */
function readFile(bytes: ArrayBuffer, isAlphaRead: boolean = false): IDdsFile {
  const read: IDdsRead = readDdsFile(bytes, isAlphaRead);

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
    expect(file.layout).toEqual({ blockBytes: 8, format: RGB_S3TC_DXT1_Format, kind: EDdsLayout.BLOCK });
  });

  it("keeps dxt1's alpha bit for a surface that reads alpha, and drops it otherwise", () => {
    // Not applied always: the transparent-black block mode occurs in files authored opaque, and reading those as rgba
    // would punch holes in surfaces the engine draws solid.
    expect(readFile(mockDdsFile({ fourCC: "DXT1" }), true).layout).toMatchObject({ format: RGBA_S3TC_DXT1_Format });
    expect(readFile(mockDdsFile({ fourCC: "DXT1" }), false).layout).toMatchObject({ format: RGB_S3TC_DXT1_Format });
  });

  it("hands out block bytes as a view rather than a copy", () => {
    // The blocks go to the gpu exactly as stored, and copying them would double what a texture costs on the way in.
    const bytes: ArrayBuffer = mockDdsFile({ fourCC: "DXT5" });
    const file: IDdsFile = readFile(bytes);

    expect(file.mipmaps[0].data.buffer).toBe(bytes);
  });
});

describe("dx10 layouts the example loader refused", () => {
  // The single commonest layout this viewer used to refuse: plain DXT5 wearing a DX10 header, which is 3,679 files of
  // the project's own resource pack and which three uploads natively.
  it("reads BC3 under a dx10 header as dxt5", () => {
    const file: IDdsFile = readFile(mockDx10DdsFile(77));

    expect(file.layout).toEqual({ blockBytes: 16, format: RGBA_S3TC_DXT5_Format, kind: EDdsLayout.BLOCK });
  });

  it("reads BC7, which the reference trees ship thirty of", () => {
    expect(readFile(mockDx10DdsFile(98)).layout).toMatchObject({ format: RGBA_BPTC_Format });
  });

  it("reads BC5, which is how a bump pair is stored", () => {
    // Worse than cosmetic before: the png fallback is deliberately skipped for bump pairs, so an ATI2 bump was
    // silently unshaded rather than degraded.
    expect(readFile(mockDx10DdsFile(83)).layout).toMatchObject({ format: RED_GREEN_RGTC2_Format });
    expect(readFile(mockDdsFile({ fourCC: "ATI2" })).layout).toMatchObject({ format: RED_GREEN_RGTC2_Format });
  });

  it("refuses a dxgi code it does not model, and says which", () => {
    // The category is what a caller matches on; the detail is what a report shows a person.
    expect(refusalOf(mockDx10DdsFile(1))).toEqual({
      detail: "DXGI_FORMAT 1 is not modelled",
      reason: EDdsRefusal.UNSUPPORTED_DXGI,
    });
  });

  it("refuses a texture array, which a surface has no way to draw", () => {
    expect(refusalOf(mockDx10DdsFile(77, { arraySize: 6 })).reason).toBe(EDdsRefusal.UNSUPPORTED_DIMENSION);
  });
});

describe("uncompressed layouts", () => {
  it("expands bgra into three's own rgba order", () => {
    const bytes: ArrayBuffer = mockUncompressedDdsFile({ height: 1, texels: [[10, 20, 30, 40]], width: 1 });
    const file: IDdsFile = readFile(bytes);

    expect(file.layout).toEqual({ channels: EDdsChannels.BGRA, kind: EDdsLayout.TEXELS });
    // Stored blue, green, red, alpha; read back red first.
    expect(Array.from(file.mipmaps[0].data)).toEqual([30, 20, 10, 40]);
  });

  it("takes an rgba ordered file, which the example loader refused outright", () => {
    // Anomaly ships 24 references to `A8B8G8R8`. Its red sits in the low byte, so the loader matched neither of its
    // two uncompressed branches - and the expansion is a copy rather than a reorder.
    const bytes: ArrayBuffer = mockUncompressedDdsFile({
      blueMask: 0x00ff0000,
      height: 1,
      redMask: 0x000000ff,
      texels: [[10, 20, 30, 40]],
      width: 1,
    });

    expect(readFile(bytes).layout).toEqual({ channels: EDdsChannels.RGBA, kind: EDdsLayout.TEXELS });
    expect(Array.from(readFile(bytes).mipmaps[0].data)).toEqual([10, 20, 30, 40]);
  });

  it("refuses a layout whose channels are not whole bytes, and names the masks", () => {
    // `R5G6B5`: unpacking it is decoding rather than reordering, which is the backend's job.
    const refusal: IDdsRefusal = refusalOf(
      mockUncompressedDdsFile({ bitCount: 16, blueMask: 0x001f, redMask: 0xf800 })
    );

    expect(refusal.reason).toBe(EDdsRefusal.UNSUPPORTED_MASKS);
    expect(refusal.detail).toContain("16 bit");
  });
});

describe("refusals", () => {
  it("tells a file that is not a dds from one that stops short", () => {
    // The two used to look identical from outside, and they are opposite fixes.
    expect(refusalOf(new Uint8Array([1, 2, 3, 4]).buffer).reason).toBe(EDdsRefusal.TRUNCATED);

    const notADds: ArrayBuffer = mockDdsFile();

    new Int32Array(notADds)[0] = 0;

    expect(refusalOf(notADds).reason).toBe(EDdsRefusal.NOT_A_DDS);
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

  it("refuses a four character tag it does not model, and says which", () => {
    const refusal: IDdsRefusal = refusalOf(mockDdsFile({ fourCC: "YUY2" }));

    expect(refusal.reason).toBe(EDdsRefusal.UNSUPPORTED_FOURCC);
    expect(refusal.detail).toContain("'YUY2'");
  });

  it("refuses a file that stops before the texels its header declares", () => {
    // A view over a buffer that is one block short throws; refusing it is what keeps a bad file from taking the app.
    const complete: ArrayBuffer = mockDdsFile({ fourCC: "DXT5", height: 8, mipmapCount: 1, width: 8 });

    expect(refusalOf(complete.slice(0, complete.byteLength - 1)).reason).toBe(EDdsRefusal.TRUNCATED);
  });

  // The defect this exists for: `compressedTexImage2D` answers `INVALID_OPERATION` for a block compressed level
  // narrower than its block, and a texture that failed to upload samples as opaque black. Anomaly ships seven leaf
  // sprays as 2x2 fully transparent DXT1 files to switch that geometry off, and every one of them drew as a solid
  // black card instead of vanishing. Direct3D takes them, which is why the game does not show this.
  it("refuses a block compressed picture smaller than one block, so the backend expands it instead", () => {
    const read: IDdsRead = readDdsFile(mockDdsFile({ fourCC: "DXT1", height: 2, width: 2 }), true);

    expect(read.file).toBeNull();
    expect(read.refusal?.reason).toBe(EDdsRefusal.SUB_BLOCK);
    expect(read.refusal?.detail).toContain("2x2");
  });

  it("takes a picture of exactly one block", () => {
    expect(readFile(mockDdsFile({ fourCC: "DXT1", height: 4, width: 4 })).width).toBe(4);
  });
});
