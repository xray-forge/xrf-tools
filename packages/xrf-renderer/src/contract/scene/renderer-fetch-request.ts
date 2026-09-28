/**
 * A request the renderer makes itself, so a texture's bytes never cross the page: a `POST` of its body with its
 * headers, which carry whatever the consumer's server asks for.
 */
export interface IRendererFetchRequest {
  url: string;
  headers: Record<string, string>;
  body: string;
}
