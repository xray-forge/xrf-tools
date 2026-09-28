import { Nullable } from "@xrf/types";
import { Texture } from "three/webgpu";

import { IRendererTextureFetch } from "#/contract/scene/renderer-texture-fetch";

/** What fetching a texture came to: the texture, or none, and what the consumer is told of it. */
export interface IRendererTextureLoad {
  texture: Nullable<Texture>;
  fetch: IRendererTextureFetch;
}
