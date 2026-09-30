/**
 * The block compression a file stores, named by the file rather than by any graphics api.
 */
export enum EDdsBlockFormat {
  /** DXT1: colour with one bit of alpha, which the surface decides whether to read. */
  BC1 = "bc1",
  /** DXT2 and DXT3: explicit four bit alpha. */
  BC2 = "bc2",
  /** DXT4 and DXT5: interpolated alpha. */
  BC3 = "bc3",
  /** One unsigned plane, as ATI1 or BC4U. */
  BC4 = "bc4",
  BC4_SIGNED = "bc4Signed",
  /** Two unsigned planes, as ATI2 or BC5U. */
  BC5 = "bc5",
  BC5_SIGNED = "bc5Signed",
  /** Unsigned floating point colour. */
  BC6H = "bc6h",
  BC6H_SIGNED = "bc6hSigned",
  BC7 = "bc7",
}

/** Texels a block covers on each side, which is also the smallest base level a gpu api takes. */
export const DDS_BLOCK_SIZE: number = 4;

/** Bytes one block of the s3tc and rgtc single-plane formats occupies. */
export const DDS_HALF_BLOCK_BYTES: number = 8;

/** Bytes one block of every other block format occupies. */
export const DDS_FULL_BLOCK_BYTES: number = 16;
