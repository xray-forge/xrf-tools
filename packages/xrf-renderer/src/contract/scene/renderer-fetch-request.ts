import { IRendererFetchBatch } from "#/contract/scene/renderer-fetch-batch";

/**
 * A request the renderer makes itself, so a texture's bytes never cross the page: a `POST` of its body with its
 * headers, which carry whatever the consumer's server asks for.
 */
export interface IRendererFetchRequest {
  url: string;
  headers: Record<string, string>;
  body: string;
  /** Where it may be sent together with others, a few requests carrying a level's thousand textures; alone without. */
  batch?: IRendererFetchBatch;
}
