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
  /** Block compressed and smaller than one block, which neither WebGL nor WebGPU takes; the backend expands it. */
  SUB_BLOCK = "subBlock",
}

/** Why one file was refused, in the terms whoever reports it needs. */
export interface IDdsRefusal {
  reason: EDdsRefusal;
  /** What was actually in the header, so a report can name the layout rather than only its category. */
  detail: string;
}
