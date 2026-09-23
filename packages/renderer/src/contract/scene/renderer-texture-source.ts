/**
 * How a texture's bytes are encoded.
 */
export enum ERendererTextureEncoding {
  /** A dds file as the game ships it; uploaded as stored. */
  DDS = "dds",
  /** A picture a browser decodes, such as the backend's png of a dds file the reader refuses. */
  IMAGE = "image",
  /** Raw rgba bytes, four a texel, top row first, for a picture a consumer made itself. */
  RGBA = "rgba",
}

/**
 * The bytes of one texture, handed over rather than copied.
 */
export type TRendererTextureSource =
  | { encoding: ERendererTextureEncoding.DDS; bytes: ArrayBuffer }
  | { encoding: ERendererTextureEncoding.IMAGE; bytes: ArrayBuffer; type: string }
  | {
      encoding: ERendererTextureEncoding.RGBA;
      bytes: ArrayBuffer;
      width: number;
      height: number;
      /** Sampled nearest rather than linear, so a pattern of texels stays crisp. */
      isNearest?: boolean;
    };
