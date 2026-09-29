/**
 * A volume texture's top level, decoded to four bytes a texel, slice after slice, each top row first.
 */
export interface IDdsVolume {
  width: number;
  height: number;
  depth: number;
  rgba: Uint8Array;
}
