/**
 * Minimal DDS files, built byte by byte.
 */

const DDS_MAGIC: number = 0x20534444;
/** Header size in bytes as the format declares it, which is also what a reader adds 4 to for its data offset. */
const HEADER_SIZE: number = 124;
const DATA_OFFSET: number = HEADER_SIZE + 4;
/** Bytes the `DX10` extended header adds before pixel data. */
const DX10_HEADER_SIZE: number = 20;
/** `DDSD_MIPMAPCOUNT`, without which a reader ignores the mipmap count and assumes one. */
const FLAG_MIPMAP_COUNT: number = 0x20000;
/** `D3D10_RESOURCE_DIMENSION_TEXTURE2D`, which is what a surface texture declares. */
const TEXTURE_2D: number = 3;

const OFFSET_MAGIC: number = 0;
const OFFSET_SIZE: number = 1;
const OFFSET_FLAGS: number = 2;
const OFFSET_HEIGHT: number = 3;
const OFFSET_WIDTH: number = 4;
const OFFSET_MIPMAP_COUNT: number = 7;
const OFFSET_FOUR_CC: number = 21;
const OFFSET_RGB_BIT_COUNT: number = 22;
const OFFSET_RED_MASK: number = 23;
const OFFSET_GREEN_MASK: number = 24;
const OFFSET_BLUE_MASK: number = 25;
const OFFSET_ALPHA_MASK: number = 26;
/** First int past the header, where the `DX10` extended header begins. */
const OFFSET_DXGI_FORMAT: number = 32;
const OFFSET_RESOURCE_DIMENSION: number = 33;
const OFFSET_ARRAY_SIZE: number = 35;

/** Bytes one compressed block occupies, by the fourCC that declares it. */
const BLOCK_BYTES: Record<string, number> = {
  ATI1: 8,
  ATI2: 16,
  DX10: 16,
  DXT1: 8,
  DXT3: 16,
  DXT5: 16,
};

/** What an unmodelled tag costs, since nothing reads its payload. */
const DEFAULT_BLOCK_BYTES: number = 16;

export interface IMockDdsOptions {
  fourCC?: string;
  width?: number;
  height?: number;
  mipmapCount?: number;
}

export interface IMockDx10DdsOptions {
  /** Bytes one block occupies, which differs between the `BC` families. */
  blockBytes?: number;
  /** Above one this is a texture array, which a surface has no way to draw. */
  arraySize?: number;
  resourceDimension?: number;
}

export interface IMockUncompressedDdsOptions {
  width?: number;
  height?: number;
  bitCount?: number;
  redMask?: number;
  greenMask?: number;
  blueMask?: number;
  alphaMask?: number;
  /** Texels to write, in the byte order the masks declare. Left out, the payload is zeroes. */
  texels?: ReadonlyArray<ReadonlyArray<number>>;
}

/** A fourCC as the little-endian int the header stores. */
function toFourCC(fourCC: string): number {
  return (
    fourCC.charCodeAt(0) | (fourCC.charCodeAt(1) << 8) | (fourCC.charCodeAt(2) << 16) | (fourCC.charCodeAt(3) << 24)
  );
}

/**
 * Bytes a compressed mip chain occupies.
 *
 * Sized rather than guessed because a reader builds each mip as a view over the buffer, so a buffer one byte short is
 * a refusal instead of a texture.
 */
function toCompressedDataSize(width: number, height: number, mipmapCount: number, blockBytes: number): number {
  let size: number = 0;
  let mipWidth: number = width;
  let mipHeight: number = height;

  for (let mip: number = 0; mip < mipmapCount; mip += 1) {
    size += (Math.max(4, mipWidth) / 4) * (Math.max(4, mipHeight) / 4) * blockBytes;

    mipWidth = Math.max(mipWidth >> 1, 1);
    mipHeight = Math.max(mipHeight >> 1, 1);
  }

  return size;
}

/**
 * A DDS file carrying a block-compressed format.
 *
 * @param options - Format, dimensions and mip count to declare.
 * @returns The file as bytes.
 */
export function mockDdsFile(options: IMockDdsOptions = {}): ArrayBuffer {
  const { fourCC = "DXT1", width = 4, height = 4, mipmapCount = 1 } = options;

  const blockBytes: number = BLOCK_BYTES[fourCC] ?? DEFAULT_BLOCK_BYTES;
  const dataSize: number = toCompressedDataSize(width, height, mipmapCount, blockBytes);
  const buffer: ArrayBuffer = new ArrayBuffer(DATA_OFFSET + dataSize);
  const header: Int32Array = new Int32Array(buffer);

  header[OFFSET_MAGIC] = DDS_MAGIC;
  header[OFFSET_SIZE] = HEADER_SIZE;
  header[OFFSET_FLAGS] = FLAG_MIPMAP_COUNT;
  header[OFFSET_HEIGHT] = height;
  header[OFFSET_WIDTH] = width;
  header[OFFSET_MIPMAP_COUNT] = mipmapCount;
  header[OFFSET_FOUR_CC] = toFourCC(fourCC);

  return buffer;
}

/**
 * A DDS file whose format is declared by a `DX10` extended header.
 *
 * @param dxgiFormat - `DXGI_FORMAT` code to declare, such as 98 for `BC7_UNORM` or 77 for `BC3_UNORM`.
 * @param options - Block size and the resource shape to declare.
 * @returns The file as bytes.
 */
export function mockDx10DdsFile(dxgiFormat: number, options: IMockDx10DdsOptions = {}): ArrayBuffer {
  const { blockBytes = BLOCK_BYTES.DX10, arraySize = 1, resourceDimension = TEXTURE_2D } = options;

  const dataSize: number = toCompressedDataSize(4, 4, 1, blockBytes);
  const buffer: ArrayBuffer = new ArrayBuffer(DATA_OFFSET + DX10_HEADER_SIZE + dataSize);
  const header: Int32Array = new Int32Array(buffer);

  header[OFFSET_MAGIC] = DDS_MAGIC;
  header[OFFSET_SIZE] = HEADER_SIZE;
  header[OFFSET_HEIGHT] = 4;
  header[OFFSET_WIDTH] = 4;
  header[OFFSET_FOUR_CC] = toFourCC("DX10");
  header[OFFSET_DXGI_FORMAT] = dxgiFormat;
  header[OFFSET_RESOURCE_DIMENSION] = resourceDimension;
  header[OFFSET_ARRAY_SIZE] = arraySize;

  return buffer;
}

/**
 * A DDS file storing uncompressed pixels, with the channel order the masks say.
 *
 * The masks are the point: the same 32 bits per pixel is `A8R8G8B8` when red sits in `0x00ff0000` and `A8B8G8R8`
 * when it sits in `0x000000ff`, and a reader has to tell them apart rather than take one for the other.
 *
 * @param options - Dimensions, the four channel masks, and the texels to write.
 * @returns The file as bytes.
 */
export function mockUncompressedDdsFile(options: IMockUncompressedDdsOptions = {}): ArrayBuffer {
  const {
    width = 4,
    height = 4,
    bitCount = 32,
    redMask = 0x00ff0000,
    greenMask = 0x0000ff00,
    blueMask = 0x000000ff,
    alphaMask = 0xff000000,
    texels,
  } = options;

  const stride: number = Math.ceil(bitCount / 8);
  const buffer: ArrayBuffer = new ArrayBuffer(DATA_OFFSET + width * height * stride);
  const header: Int32Array = new Int32Array(buffer);

  header[OFFSET_MAGIC] = DDS_MAGIC;
  header[OFFSET_SIZE] = HEADER_SIZE;
  header[OFFSET_HEIGHT] = height;
  header[OFFSET_WIDTH] = width;
  header[OFFSET_FOUR_CC] = 0;
  header[OFFSET_RGB_BIT_COUNT] = bitCount;
  header[OFFSET_RED_MASK] = redMask;
  header[OFFSET_GREEN_MASK] = greenMask;
  header[OFFSET_BLUE_MASK] = blueMask;
  header[OFFSET_ALPHA_MASK] = alphaMask;

  if (texels) {
    const payload: Uint8Array = new Uint8Array(buffer, DATA_OFFSET);

    texels.forEach((texel: ReadonlyArray<number>, index: number) => {
      texel.forEach((channel: number, at: number) => {
        payload[index * stride + at] = channel;
      });
    });
  }

  return buffer;
}

/**
 * A DDS file in a layout the frontend reader does not model, which is what sends a texture to the backend to decode.
 *
 * `R5G6B5`: sixteen bits whose channels are not whole bytes, so expanding it is decoding rather than reordering. Named
 * rather than spelled out at each call, because what the masks are does not matter to a test about the fallback - only
 * that the reader declines them - and the set of declined layouts shrinks as the reader learns more of them.
 */
export function mockUndecodableDdsFile(): ArrayBuffer {
  return mockUncompressedDdsFile({ alphaMask: 0, bitCount: 16, blueMask: 0x001f, greenMask: 0x07e0, redMask: 0xf800 });
}
