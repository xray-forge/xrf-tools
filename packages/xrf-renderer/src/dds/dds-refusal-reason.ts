/** Why a dds file cannot be uploaded as it is stored. */
export enum EDdsRefusalReason {
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
  /** A cubemap the reader does not take: missing faces, or faces of uncompressed texels. */
  CUBEMAP = "cubemap",
  /** A texture array or a volume, which a surface has no way to draw either. */
  UNSUPPORTED_DIMENSION = "unsupportedDimension",
  /** Block compressed with a side that is not whole blocks, which WebGPU refuses at its base; the backend expands it. */
  UNALIGNED_BLOCKS = "unalignedBlocks",
}
