import { Nullable } from "@xrf/types";

import { toDdsFourCc } from "#/texture/dds/dds-fourcc";
import { IDdsChannelMasks } from "#/texture/dds/dds-masks";
import { EDdsRefusal, IDdsRefusal } from "#/texture/dds/dds-refusal";

/** `DDS `, little endian. */
const DDS_MAGIC: number = 0x20534444;
/** The header is thirty-one `u32` including the magic, and the `DX10` extension five more. */
const HEADER_INTS: number = 31;
const EXTENDED_HEADER_INTS: number = 5;

/** `DDSD_MIPMAPCOUNT`, which says the mip count field means anything. */
const DDSD_MIPMAPCOUNT: number = 0x20000;

/** `DDSCAPS2_CUBEMAP` and the six faces that have to accompany it. */
const DDSCAPS2_CUBEMAP: number = 0x200;
const CUBEMAP_FACES: ReadonlyArray<number> = [0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000];

/** Offsets into the header, counted in `u32`. */
const OFF_MAGIC: number = 0;
const OFF_SIZE: number = 1;
const OFF_FLAGS: number = 2;
const OFF_HEIGHT: number = 3;
const OFF_WIDTH: number = 4;
const OFF_MIPMAP_COUNT: number = 7;
const OFF_PF_FOURCC: number = 21;
const OFF_RGB_BIT_COUNT: number = 22;
const OFF_R_MASK: number = 23;
const OFF_G_MASK: number = 24;
const OFF_B_MASK: number = 25;
const OFF_A_MASK: number = 26;
const OFF_CAPS2: number = 28;

/** Offsets into the `DX10` extended header, counted in `u32` from its own start. */
const OFF_DXGI_FORMAT: number = 0;
const OFF_RESOURCE_DIMENSION: number = 1;
const OFF_ARRAY_SIZE: number = 3;

/** `D3D10_RESOURCE_DIMENSION_TEXTURE3D`, a volume, which no surface draws. */
export const DDS_DIMENSION_TEXTURE_3D: number = 4;

/** What the `DX10` extended header says about the resource. */
export interface IDdsExtendedHeader {
  dxgiFormat: number;
  dimension: number;
  arraySize: number;
}

/** A dds header, as the fields a read decides on. */
export interface IDdsHeader {
  width: number;
  height: number;
  mipmapCount: number;
  /** The pixel format's four character tag, or empty for a layout stored as channel masks. */
  fourCc: string;
  masks: IDdsChannelMasks;
  /** The extended header, for a file whose tag is `DX10`. */
  extended: Nullable<IDdsExtendedHeader>;
  /** Whether the caps call the file a cubemap, and whether all six faces accompany the claim. */
  cubemap: Nullable<{ isWhole: boolean }>;
  /** Where the first mip starts. */
  dataOffset: number;
}

/** What a header read came to: exactly one of the two is present. */
export interface IDdsHeaderRead {
  header: Nullable<IDdsHeader>;
  refusal: Nullable<IDdsRefusal>;
}

/**
 * Reads a dds header, or says why the file has none worth reading.
 *
 * @param bytes - The file as read.
 * @returns The header, or the reason it could not be read.
 */
export function readDdsHeader(bytes: ArrayBuffer): IDdsHeaderRead {
  if (bytes.byteLength < HEADER_INTS * 4) {
    return refuse(EDdsRefusal.TRUNCATED, `the file is ${bytes.byteLength} bytes, short of a dds header`);
  }

  // Unsigned, because a channel mask reaches the top bit: read as `Int32Array`, `0xff000000` comes back negative
  // and compares equal to nothing, which takes every `A8B8G8R8` file for an unmodelled layout.
  const words: Uint32Array = new Uint32Array(bytes, 0, HEADER_INTS);

  if (words[OFF_MAGIC] !== DDS_MAGIC) {
    return refuse(EDdsRefusal.NOT_A_DDS, "the file does not open with the 'DDS ' magic number");
  }

  // Read off the tag rather than off `DDPF_FOURCC`: a pixel format storing channel masks leaves the tag zero, and
  // trusting the flag instead would refuse the files in the reference trees that set the tag without it. `DDSLoader`
  // switches on the tag for the same reason, and 41,040 files agree with it.
  const fourCc: string = words[OFF_PF_FOURCC] === 0 ? "" : toDdsFourCc(words[OFF_PF_FOURCC]);
  // The header states its own size, and the extension follows it.
  let dataOffset: number = words[OFF_SIZE] + 4;
  let extended: Nullable<IDdsExtendedHeader> = null;

  if (fourCc === "DX10") {
    const at: number = (HEADER_INTS + 1) * 4;

    if (bytes.byteLength < at + EXTENDED_HEADER_INTS * 4) {
      return refuse(EDdsRefusal.TRUNCATED, "the file declares a DX10 header and stops before it");
    }

    const extendedWords: Uint32Array = new Uint32Array(bytes, at, EXTENDED_HEADER_INTS);

    extended = {
      arraySize: extendedWords[OFF_ARRAY_SIZE],
      dimension: extendedWords[OFF_RESOURCE_DIMENSION],
      dxgiFormat: extendedWords[OFF_DXGI_FORMAT],
    };
    dataOffset += EXTENDED_HEADER_INTS * 4;
  }

  const caps2: number = words[OFF_CAPS2];

  return {
    header: {
      cubemap: caps2 & DDSCAPS2_CUBEMAP ? { isWhole: CUBEMAP_FACES.every((face: number) => caps2 & face) } : null,
      dataOffset,
      extended,
      fourCc,
      height: words[OFF_HEIGHT],
      masks: {
        alpha: words[OFF_A_MASK],
        bitCount: words[OFF_RGB_BIT_COUNT],
        blue: words[OFF_B_MASK],
        green: words[OFF_G_MASK],
        red: words[OFF_R_MASK],
      },
      mipmapCount: words[OFF_FLAGS] & DDSD_MIPMAPCOUNT ? Math.max(1, words[OFF_MIPMAP_COUNT]) : 1,
      width: words[OFF_WIDTH],
    },
    refusal: null,
  };
}

function refuse(reason: EDdsRefusal, detail: string): IDdsHeaderRead {
  return { header: null, refusal: { detail, reason } };
}
