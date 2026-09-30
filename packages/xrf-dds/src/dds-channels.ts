/**
 * The byte order a texel-stored layout keeps its channels in.
 */
export enum EDdsChannels {
  /** What `D3DFMT_A8R8G8B8` stores, which is most of what the sdk wrote uncompressed. */
  BGRA = "bgra",
  /** `D3DFMT_R8G8B8`, three bytes a texel, opaque once expanded. */
  BGR = "bgr",
  /** Already rgba, so expansion is a copy. */
  RGBA = "rgba",
}

/** Bytes one texel occupies in the file, which is not what it occupies once expanded. */
export function getDdsSourceStride(channels: EDdsChannels): number {
  return channels === EDdsChannels.BGR ? 3 : 4;
}
