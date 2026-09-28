import { IRendererFetchRequest } from "#/contract/scene/renderer-fetch-request";

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
  /** A dds file the renderer fetches itself, and the picture it falls back to where the file cannot be read. */
  FETCH = "fetch",
}

/**
 * The bytes of one texture, handed over rather than copied, or where the renderer fetches them.
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
    }
  | {
      encoding: ERendererTextureEncoding.FETCH;
      /** The file as stored. */
      file: IRendererFetchRequest;
      /** A picture of it, fetched only where the file is a layout the reader refuses. */
      picture: IRendererFetchRequest;
    };
