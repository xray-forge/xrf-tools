/** A bulk call as the request that makes it: everything a `fetch` needs, on whichever thread makes it. */
export interface IBulkRequest {
  url: string;
  headers: Record<string, string>;
  /** The call's arguments as JSON. */
  body: string;
}
