import { AnyPixelFormat, TextureDataType } from "three/webgpu";

/** One colour a target writes. */
export interface IColourAttachment {
  /** What the device labels it. */
  name: string;
  /** Four channels unless said. */
  format?: AnyPixelFormat;
  /** Half floats unless said. */
  type?: TextureDataType;
  /** Whether it is sampled between texels; read texel by texel otherwise. */
  isFiltered?: boolean;
}
