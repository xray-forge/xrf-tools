import { Nullable } from "@xrf/types";

import { IRendererTextureSize } from "#/contract/scene/renderer-texture-size";

/** What a fetched texture came to, told once its fetch settles. */
export interface IRendererTextureFetch {
  /** Its size as uploaded, or null where nothing was. */
  size: Nullable<IRendererTextureSize>;
  /** Whether it is the source's picture, fetched because its file could not be read as stored. */
  isDecoded: boolean;
  /** Why nothing was uploaded, or null where something was. */
  failure: Nullable<string>;
  /** Bytes its fetches answered. */
  bytes: number;
  /** Milliseconds its fetches took. */
  duration: number;
}
