import { Nullable } from "@xrf/types";

/** One answered request: its bytes, and what they are. */
export interface IRendererFetchedBytes {
  bytes: ArrayBuffer;
  type: Nullable<string>;
}
