/** A bulk call as the request that makes it: everything a `fetch` needs, on whichever thread makes it. */
export interface IBulkRequest {
  url: string;
  headers: Record<string, string>;
  /** The call's arguments as JSON. */
  body: string;
  /**
   * Where it may be posted together with other calls, and as what: the transport's batch route answers each call by
   * its part as it finishes, framed as the renderer's `IRendererFetchBatch` says.
   */
  batch?: { url: string; call: string };
}
