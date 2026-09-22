import { Nullable } from "@xrf/types";

import {
  EDdsChannels,
  EDdsLayout,
  getDdsDxgiLayout,
  getDdsFourCcLayout,
  getDdsMaskLayout,
  getDdsSourceStride,
  IDdsChannelMasks,
  TDdsLayout,
} from "@/core/render/lib/dds/dds-format";

/** Why a dds file cannot be uploaded as it is stored. */
export enum EDdsRefusal {
  /** The magic number is not `DDS `, so this is not the container at all. */
  NOT_A_DDS = "notADds",
  /** The file stops short of the header, or of the texels the header declares. */
  TRUNCATED = "truncated",
  /** A `DDPF_FOURCC` tag this does not model. */
  UNSUPPORTED_FOURCC = "unsupportedFourCc",
  /** A `DXGI_FORMAT` this does not model. */
  UNSUPPORTED_DXGI = "unsupportedDxgi",
  /** An uncompressed layout whose channels are not whole bytes, which the backend expands instead. */
  UNSUPPORTED_MASKS = "unsupportedMasks",
  /** Six faces rather than one picture, which no surface of a level or a model draws. */
  CUBEMAP = "cubemap",
  /** A texture array or a volume, which a surface has no way to draw either. */
  UNSUPPORTED_DIMENSION = "unsupportedDimension",
  /** Block compressed and smaller than one block, which WebGL will not take, so the backend expands it instead. */
  SUB_BLOCK = "subBlock",
}

/** Why one file was refused, in the terms whoever reports it needs. */
export interface IDdsRefusal {
  reason: EDdsRefusal;
  /** What was actually in the header, so a report can name the layout rather than only its category. */
  detail: string;
}

/** Texels a block compressed level covers on each side, which is also the smallest level WebGL will take. */
const DDS_BLOCK_SIZE: number = 4;

/** One mip as it will be uploaded: block bytes untouched, or texels already expanded to rgba. */
export interface IDdsMipmap {
  data: Uint8Array;
  width: number;
  height: number;
}

/** A dds file, read. */
export interface IDdsFile {
  width: number;
  height: number;
  /** Mips in order, largest first. */
  mipmaps: Array<IDdsMipmap>;
  mipmapCount: number;
  layout: TDdsLayout;
}

/**
 * What a read came to: exactly one of the two is present.
 */
export interface IDdsRead {
  file: Nullable<IDdsFile>;
  refusal: Nullable<IDdsRefusal>;
}

/** `DDS `, little endian. */
const DDS_MAGIC: number = 0x20534444;
/** The header is thirty-one `u32` including the magic, and the `DX10` extension five more. */
const HEADER_INTS: number = 31;
const EXTENDED_HEADER_INTS: number = 5;

/** `DDSD_MIPMAPCOUNT`, which says the mip count field means anything. */
const DDSD_MIPMAPCOUNT: number = 0x20000;
/** `D3D10_RESOURCE_DIMENSION_TEXTURE3D`, a volume, which no surface draws. */
const TEXTURE_3D: number = 4;

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

/** Texels expand to four bytes however few the file stored. */
const EXPANDED_STRIDE: number = 4;

/**
 * Reads a dds file, or says why it cannot be uploaded as stored.
 *
 * @param bytes - The file as read.
 * @param isAlphaRead - Whether any surface drawn with this file samples its alpha channel, which is the only thing
 *   that decides how `DXT1`'s identical blocks are uploaded.
 * @returns The file, or the reason it was refused.
 */
export function readDdsFile(bytes: ArrayBuffer, isAlphaRead: boolean = false): IDdsRead {
  if (bytes.byteLength < HEADER_INTS * 4) {
    return refuse(EDdsRefusal.TRUNCATED, `the file is ${bytes.byteLength} bytes, short of a dds header`);
  }

  // Unsigned, because a channel mask reaches the top bit: read as `Int32Array`, `0xff000000` comes back negative
  // and compares equal to nothing, which takes every `A8B8G8R8` file for an unmodelled layout.
  const header: Uint32Array = new Uint32Array(bytes, 0, HEADER_INTS);

  if (header[OFF_MAGIC] !== DDS_MAGIC) {
    return refuse(EDdsRefusal.NOT_A_DDS, "the file does not open with the 'DDS ' magic number");
  }

  // Read off the tag rather than off `DDPF_FOURCC`: a pixel format storing channel masks leaves the tag zero, which
  // matches nothing below, and trusting the flag instead would refuse the files in the reference trees that set the
  // tag without it. `DDSLoader` switches on the tag for the same reason, and 41,040 files agree with it.
  const fourCc: string = toFourCc(header[OFF_PF_FOURCC]);
  const isFourCc: boolean = header[OFF_PF_FOURCC] !== 0;
  const isExtended: boolean = fourCc === "DX10";

  // The header states its own size, and the extension follows it.
  let dataOffset: number = header[OFF_SIZE] + 4;
  let layout: Nullable<TDdsLayout>;

  if (isExtended) {
    const extended: Nullable<Uint32Array> = readExtendedHeader(bytes);

    if (!extended) {
      return refuse(EDdsRefusal.TRUNCATED, "the file declares a DX10 header and stops before it");
    }

    const dimension: number = extended[OFF_RESOURCE_DIMENSION];
    const arraySize: number = extended[OFF_ARRAY_SIZE];

    // Only the two that would really misparse. A writer leaving either field zero means a plain 2d texture, and
    // refusing those would refuse files that read correctly.
    if (dimension === TEXTURE_3D || arraySize > 1) {
      return refuse(
        EDdsRefusal.UNSUPPORTED_DIMENSION,
        `the file is a DX10 resource of dimension ${dimension} and array size ${arraySize}`
      );
    }

    dataOffset += EXTENDED_HEADER_INTS * 4;
    layout = getDdsDxgiLayout(extended[OFF_DXGI_FORMAT], isAlphaRead);

    if (!layout) {
      return refuse(EDdsRefusal.UNSUPPORTED_DXGI, `DXGI_FORMAT ${extended[OFF_DXGI_FORMAT]} is not modelled`);
    }
  } else if (isFourCc) {
    layout = getDdsFourCcLayout(fourCc, isAlphaRead);

    if (!layout) {
      return refuse(EDdsRefusal.UNSUPPORTED_FOURCC, `the four character tag '${fourCc}' is not modelled`);
    }
  } else {
    const masks: IDdsChannelMasks = {
      alpha: header[OFF_A_MASK],
      bitCount: header[OFF_RGB_BIT_COUNT],
      blue: header[OFF_B_MASK],
      green: header[OFF_G_MASK],
      red: header[OFF_R_MASK],
    };

    layout = getDdsMaskLayout(masks);

    if (!layout) {
      return refuse(EDdsRefusal.UNSUPPORTED_MASKS, describeMasks(masks));
    }
  }

  const caps2: number = header[OFF_CAPS2];

  // Refused rather than read: a cubemap is six faces, and drawing one flat would stretch whichever face came first
  // over the surface. Whether its faces are all there is said in the detail, because a malformed cubemap is a broken
  // file while a whole one is simply the wrong kind of picture.
  if (caps2 & DDSCAPS2_CUBEMAP) {
    const isWhole: boolean = CUBEMAP_FACES.every((face: number) => caps2 & face);

    return refuse(EDdsRefusal.CUBEMAP, `the file is a cubemap, ${isWhole ? "six faces" : "missing faces"}`);
  }

  const width: number = header[OFF_WIDTH];
  const height: number = header[OFF_HEIGHT];
  const mipmapCount: number = header[OFF_FLAGS] & DDSD_MIPMAPCOUNT ? Math.max(1, header[OFF_MIPMAP_COUNT]) : 1;

  // Refused rather than uploaded: `compressedTexImage2D` answers `INVALID_OPERATION` for a block compressed level
  // narrower than its block, and a texture that failed to upload samples as opaque black - which is a surface drawn
  // solid where its file said to draw nothing at all. Direct3D takes these, which is why the game shows them and a
  // browser does not. The backend expands them to png instead, where the alpha survives.
  if (layout.kind === EDdsLayout.BLOCK && (width < DDS_BLOCK_SIZE || height < DDS_BLOCK_SIZE)) {
    return refuse(EDdsRefusal.SUB_BLOCK, `the picture is ${width}x${height}, under the ${DDS_BLOCK_SIZE} of a block`);
  }

  const mipmaps: Array<IDdsMipmap> | EDdsRefusal.TRUNCATED = readMipmaps(bytes, dataOffset, {
    height,
    layout,
    mipmapCount,
    width,
  });

  if (mipmaps === EDdsRefusal.TRUNCATED) {
    return refuse(EDdsRefusal.TRUNCATED, "the file stops before the texels its header declares");
  }

  return { file: { height, layout, mipmapCount, mipmaps, width }, refusal: null };
}

/** What a mip walk needs to know about the file it is walking. */
interface IDdsShape {
  width: number;
  height: number;
  mipmapCount: number;
  layout: TDdsLayout;
}

/**
 * Walks every face's mip chain, halving until the chain is spent.
 *
 * A block layout is handed out as a view over the file rather than a copy: the blocks go to the gpu exactly as stored,
 * and copying them would double what a texture costs on the way in.
 */
function readMipmaps(bytes: ArrayBuffer, start: number, shape: IDdsShape): Array<IDdsMipmap> | EDdsRefusal.TRUNCATED {
  const mipmaps: Array<IDdsMipmap> = [];

  let offset: number = start;
  let width: number = shape.width;
  let height: number = shape.height;

  for (let level = 0; level < shape.mipmapCount; level += 1) {
    const stored: number = getStoredLength(shape.layout, width, height);

    if (offset + stored > bytes.byteLength) {
      return EDdsRefusal.TRUNCATED;
    }

    mipmaps.push({
      data:
        shape.layout.kind === EDdsLayout.BLOCK
          ? new Uint8Array(bytes, offset, stored)
          : expandTexels(bytes, offset, width, height, shape.layout.channels),
      height,
      width,
    });

    offset += stored;
    width = Math.max(width >> 1, 1);
    height = Math.max(height >> 1, 1);
  }

  return mipmaps;
}

/** Bytes one mip occupies **in the file**, which for a texel layout is not what it occupies once expanded. */
function getStoredLength(layout: TDdsLayout, width: number, height: number): number {
  if (layout.kind === EDdsLayout.BLOCK) {
    // A chain runs below one block, and the smallest levels still cost a whole one.
    return (Math.max(4, width) / 4) * (Math.max(4, height) / 4) * layout.blockBytes;
  }

  return width * height * getDdsSourceStride(layout.channels);
}

/**
 * Expands one stored mip into the rgba byte order three.js uploads.
 *
 * @param bytes - The file.
 * @param offset - Where this mip starts in it.
 * @param width - Its width in texels.
 * @param height - Its height in texels.
 * @param channels - The order the file keeps its channels in.
 * @returns Four bytes a texel, red first.
 */
function expandTexels(
  bytes: ArrayBuffer,
  offset: number,
  width: number,
  height: number,
  channels: EDdsChannels
): Uint8Array {
  const stride: number = getDdsSourceStride(channels);
  const source: Uint8Array = new Uint8Array(bytes, offset, width * height * stride);

  if (channels === EDdsChannels.RGBA) {
    // Already three's own order, so the expansion is a copy rather than a reorder.
    return new Uint8Array(source);
  }

  const expanded: Uint8Array = new Uint8Array(width * height * EXPANDED_STRIDE);
  const hasAlpha: boolean = channels === EDdsChannels.BGRA;

  for (let texel = 0; texel < width * height; texel += 1) {
    const from: number = texel * stride;
    const to: number = texel * EXPANDED_STRIDE;

    expanded[to] = source[from + 2];
    expanded[to + 1] = source[from + 1];
    expanded[to + 2] = source[from];
    // A layout storing three bytes says nothing about alpha, and a texture that read zero there would be invisible.
    expanded[to + 3] = hasAlpha ? source[from + 3] : 255;
  }

  return expanded;
}

function readExtendedHeader(bytes: ArrayBuffer): Nullable<Uint32Array> {
  const at: number = (HEADER_INTS + 1) * 4;

  if (bytes.byteLength < at + EXTENDED_HEADER_INTS * 4) {
    return null;
  }

  return new Uint32Array(bytes, at, EXTENDED_HEADER_INTS);
}

/** The four character tag a `u32` spells, low byte first. */
function toFourCc(value: number): string {
  return String.fromCharCode(value & 0xff, (value >> 8) & 0xff, (value >> 16) & 0xff, (value >> 24) & 0xff);
}

function describeMasks(masks: IDdsChannelMasks): string {
  return (
    `an uncompressed ${masks.bitCount} bit layout, ` +
    `r=${toMask(masks.red)} g=${toMask(masks.green)} b=${toMask(masks.blue)} a=${toMask(masks.alpha)}`
  );
}

function toMask(mask: number): string {
  return `0x${(mask >>> 0).toString(16).padStart(8, "0")}`;
}

function refuse(reason: EDdsRefusal, detail: string): IDdsRead {
  return { file: null, refusal: { detail, reason } };
}
