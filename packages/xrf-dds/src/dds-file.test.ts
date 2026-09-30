import { describe, expect, it } from "@jest/globals";

import { EDdsBlockFormat } from "#/dds-block-format";
import { EDdsChannels } from "#/dds-channels";
import { IDdsFile, readDdsFile } from "#/dds-file";
import { mockCubeDdsFile, mockDdsFile, mockDx10DdsFile, mockUncompressedDdsFile } from "#/dds-fixtures";
import { EDdsLayout } from "#/dds-layout";
import { IDdsMipmap } from "#/dds-mipmap";
import { IDdsRead } from "#/dds-read";
import { IDdsRefusal } from "#/dds-refusal";
import { EDdsRefusalReason } from "#/dds-refusal-reason";

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
      reason: EDdsRefusalReason.UNSUPPORTED_DXGI,
    });
    expect(refusalOf(mockDdsFile({ fourCC: "YUY2" }))).toEqual({
      detail: "the four character tag 'YUY2' is not modelled",
      reason: EDdsRefusalReason.UNSUPPORTED_FOURCC,
    });

    const masks: IDdsRefusal = refusalOf(mockUncompressedDdsFile({ bitCount: 16, blueMask: 0x001f, redMask: 0xf800 }));

    expect(masks.reason).toBe(EDdsRefusalReason.UNSUPPORTED_MASKS);
    expect(masks.detail).toContain("16 bit");
  });

  it("refuses a texture array or a volume, which a surface has no way to draw", () => {
    expect(refusalOf(mockDx10DdsFile(77, { arraySize: 6 })).reason).toBe(EDdsRefusalReason.UNSUPPORTED_DIMENSION);
    expect(refusalOf(mockDx10DdsFile(77, { resourceDimension: 4 })).reason).toBe(
      EDdsRefusalReason.UNSUPPORTED_DIMENSION
    );
  });

  // The legacy header states a volume in its caps alone, as `water_SBumpVolume` does; read, its slices would be levels.
  it("refuses a volume the legacy header states", () => {
    const volume: ArrayBuffer = mockDdsFile({ fourCC: "DXT5" });
    const words: Uint32Array = new Uint32Array(volume);

    words[6] = 16;
    words[28] = 0x200000;

    expect(refusalOf(volume)).toEqual({
      detail: "the file is a volume of 16 slices",
      reason: EDdsRefusalReason.UNSUPPORTED_DIMENSION,
    });
  });

  it("refuses a cubemap missing a face", () => {
    const partial: ArrayBuffer = mockDdsFile();

    new Uint32Array(partial)[28] = 0x200 | 0x400;

    expect(refusalOf(partial)).toEqual({
      detail: "the file is a cubemap missing faces",
      reason: EDdsRefusalReason.CUBEMAP,
    });
  });

  // A file stores each face's whole chain before the next face; a cube uploads each level's faces together.
  it("reads a cubemap's six faces regrouped by level, each face in its order", () => {
    const file: IDdsFile = readDdsFile(mockCubeDdsFile({ height: 8, mipmapCount: 2, width: 8 })).file as IDdsFile;
    const [top, next] = file.mipmaps;

    expect(file.isCube).toBe(true);
    expect(top.data.byteLength).toBe(4 * 8 * 6);
    expect(Array.from(top.data.filter((_, at: number) => at % 32 === 0))).toEqual([1, 2, 3, 4, 5, 6]);
    expect(next.width).toBe(4);
    expect(Array.from(next.data.filter((_, at: number) => at % 8 === 0))).toEqual([1, 2, 3, 4, 5, 6]);
  });

  it("refuses a file that stops before the texels its header declares", () => {
    // A view over a buffer that is one block short throws; refusing it is what keeps a bad file from taking the app.
    const complete: ArrayBuffer = mockDdsFile({ fourCC: "DXT5", height: 8, mipmapCount: 1, width: 8 });

    expect(refusalOf(complete.slice(0, complete.byteLength - 1)).reason).toBe(EDdsRefusalReason.TRUNCATED);
  });

  // The defect this exists for: WebGPU invalidates a block compressed texture whose base level is not whole blocks,
  // and a texture that failed to upload samples as black. Anomaly ships seven leaf sprays as 2x2 fully transparent
  // DXT1 files to switch that geometry off, and they drew as solid black cards. Direct3D takes them, which is why the
  // game does not show this.
  it("refuses a block compressed picture that is not whole blocks, so the backend expands it instead", () => {
    const small: IDdsRefusal = refusalOf(mockDdsFile({ fourCC: "DXT1", height: 2, width: 2 }));

    expect(small.reason).toBe(EDdsRefusalReason.UNALIGNED_BLOCKS);
    expect(small.detail).toContain("2x2");
    expect(refusalOf(mockDdsFile({ fourCC: "DXT5", height: 6, width: 6 })).reason).toBe(
      EDdsRefusalReason.UNALIGNED_BLOCKS
    );
    expect(refusalOf(mockDdsFile({ fourCC: "DXT5", height: 6, width: 12 })).reason).toBe(
      EDdsRefusalReason.UNALIGNED_BLOCKS
    );
  });

  it("reads each level of a chain that ends mid block from where the level before stopped", () => {
    const bytes: ArrayBuffer = mockDdsFile({ fourCC: "DXT5", height: 768, mipmapCount: 11, width: 1024 });
    const { mipmaps }: IDdsFile = readFile(bytes);

    expect(mipmaps.map(({ width, height }: IDdsMipmap) => `${width}x${height}`).slice(6)).toEqual([
      "16x12",
      "8x6",
      "4x3",
      "2x1",
      "1x1",
    ]);
    // 8x6 is two blocks by two, where halving the texels would read a block and a half.
    expect(mipmaps[7].data.byteLength).toBe(4 * 16);

    for (let level: number = 1; level < mipmaps.length; level += 1) {
      const before: Uint8Array = mipmaps[level - 1].data;

      expect(mipmaps[level].data.byteOffset).toBe(before.byteOffset + before.byteLength);
    }

    const last: Uint8Array = mipmaps[mipmaps.length - 1].data;

    expect(last.byteOffset + last.byteLength).toBe(bytes.byteLength);
  });

  // Literal counts, where the fixture sizes its files by the same sum the reader walks.
  it("reads a full chain as many bytes as the format stores, down to its one block levels", () => {
    function toChainBytes(file: IDdsFile): number {
      return file.mipmaps.reduce((total: number, { data }: IDdsMipmap) => total + data.byteLength, 0);
    }

    const wide: IDdsFile = readFile(mockDdsFile({ fourCC: "DXT1", height: 4, mipmapCount: 9, width: 256 }));

    expect(toChainBytes(readFile(mockDdsFile({ fourCC: "DXT5", height: 768, mipmapCount: 11, width: 1024 })))).toBe(
      1_048_624
    );
    // 256x4 to 1x1: 64, 32, 16, 8, 4, 2 blocks, then three levels of one.
    expect(toChainBytes(wide)).toBe(1_032);
    expect(wide.mipmaps.map(({ width, height }: IDdsMipmap) => `${width}x${height}`)).toEqual([
      "256x4",
      "128x2",
      "64x1",
      "32x1",
      "16x1",
      "8x1",
      "4x1",
      "2x1",
      "1x1",
    ]);
  });
});
